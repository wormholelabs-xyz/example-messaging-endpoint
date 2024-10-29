module router::router {
    use aptos_framework::aptos_hash;
    use router::bitmap;
    use router::integrator;
    use router::universal_address::{Self, UniversalAddress};
    use std::bcs;
    use std::signer;
    use std::table::{Self, Table};
    use std::vector;

    struct OutboxMessageKey has copy, drop {
        integrator_addr: address,
        sequence: u64
    }

    struct OutboxMessage has copy, drop, store {
        /// The sending integrator as a 32-byte universal address
        src_addr: UniversalAddress,
        /// The sequence number of the message
        sequence: u64,
        /// The destination chain's Wormhole Chain ID
        dst_chain: u16,
        /// The destination address as a 32-byte universal address
        dst_addr: UniversalAddress,
        /// The keccak256 of an arbitrary payload
        payload_hash: vector<u8>,
        /// The bitmap of send-enabled transceivers for this destination chain that have not picked up the message
        outstanding_transceivers: u128
    }

    struct OutboxState has key {
        outbox_messages: Table<OutboxMessageKey, OutboxMessage>
    }

    struct AttestationInfo has copy, drop, store {
        /// Replay protection flag
        executed: bool,
        /// The bitmap of receive-enabled transceivers for this source chain that have attested to the message
        attested_transceivers: u128
    }

    struct AttestationState has key {
        attestation_infos: Table<vector<u8>, AttestationInfo>
    }

    fun init_module(resource_account: &signer) {
        move_to(resource_account, OutboxState {
            outbox_messages: table::new<OutboxMessageKey, OutboxMessage>(),
        });
        move_to(resource_account, AttestationState {
            attestation_infos: table::new<vector<u8>, AttestationInfo>(),
        });
    }

    #[test_only]
    /// Initialise module for testing.
    public fun init_module_test() {
        use aptos_framework::account;
        // recover the signer for the module's account
        let signer_cap = account::create_test_signer_cap(@router);
        let signer = account::create_signer_with_capability(&signer_cap);
        // then call the initialiser
        init_module(&signer)
    }

    public fun register(integrator_acct: &signer, admin_addr: address) {
        integrator::new_integrator(integrator_acct, admin_addr);
    }

    public fun send_message(integrator_acct: &signer, dst_chain: u16, dst_addr: UniversalAddress, payload_hash: vector<u8>): u64 acquires OutboxState {
        assert!(vector::length(&payload_hash) == 32);
        let integrator_addr = signer::address_of(integrator_acct);
        // MUST have at least one enabled send Transceiver for `dstChain`.
        let outstanding_transceivers = integrator::get_enabled_send_transceivers(integrator_addr, dst_chain);
        assert!(outstanding_transceivers != 0);
        // Increments the Integrator's sequence, creates and stores the outbox item.
        // MUST set the current enabled Send Transceivers as the Outstanding Transceivers for that message.
        let src_addr = universal_address::from_address(integrator_addr);
        let sequence = integrator::use_sequence(integrator_acct);
        table::add(&mut OutboxState[@router].outbox_messages, OutboxMessageKey{integrator_addr, sequence}, OutboxMessage {
            src_addr, sequence, dst_chain, dst_addr, payload_hash, outstanding_transceivers
        });
        sequence
    }

    #[view]
    public fun get_outbox_message(integrator_addr: address, sequence: u64): (UniversalAddress, u64, u16, UniversalAddress, vector<u8>, u128) acquires OutboxState {
        let message = table::borrow(&OutboxState[@router].outbox_messages, OutboxMessageKey{integrator_addr, sequence});
        (message.src_addr, message.sequence, message.dst_chain, message.dst_addr, message.payload_hash, message.outstanding_transceivers)
    }

    public fun pick_up_message(transceiver_acct: &signer, integrator_addr: address, sequence: u64): (UniversalAddress, u64, u16, UniversalAddress, vector<u8>, u128) acquires OutboxState {
        let key = OutboxMessageKey{integrator_addr, sequence};
        let message = table::borrow(&OutboxState[@router].outbox_messages, key);
        // MUST check that the Transceiver is an enabled send Transceiver for the Integrator (`srcAddr`) and chain (`dstChain`).
        // Since the enabled transceivers are copied to the outstanding transceivers field when the outbox message is generated,
        // this just needs the corresponding index to check against.
        let transceiver_addr = signer::address_of(transceiver_acct);
        let index = integrator::get_transceiver_index(integrator_addr, transceiver_addr);
        // MUST check that the Transceiver has NOT already picked up the message.
        // Marks the Transceiver as having picked up the message.
        let new_outstanding_transceivers = bitmap::disable(message.outstanding_transceivers, index);
        // In order to reduce integrator / user costs, upon the last enabled sending Transceiver's pickup, any outgoing message state MUST be cleared.
        if (new_outstanding_transceivers == 0) {
            let message = table::remove(&mut OutboxState[@router].outbox_messages, key);
            (message.src_addr, message.sequence, message.dst_chain, message.dst_addr, message.payload_hash, new_outstanding_transceivers)
        } else {
            let mut_message = table::borrow_mut(&mut OutboxState[@router].outbox_messages, key);
            mut_message.outstanding_transceivers = new_outstanding_transceivers;
            (mut_message.src_addr, mut_message.sequence, mut_message.dst_chain, mut_message.dst_addr, mut_message.payload_hash, new_outstanding_transceivers)
        }
    }

    #[view]
    public fun compute_message_hash(src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, dst_addr: vector<u8>, payload_hash: vector<u8>): vector<u8> {
        assert!(src_addr.length() == 32);
        assert!(dst_addr.length() == 32);
        assert!(payload_hash.length() == 32);
        // MUST calculate the message digest as keccak256(abi.encodePacked(sourceChain, sourceAddress, sequence, destinationChain, destinationAddress, payloadHash))
        // we reuse the native bcs serialiser -- it uses little-endian encoding, and
        // we need big-endian, so the results are reversed
        let bytes = vector::empty();
        let v = bcs::to_bytes(&src_chain);
        v.reverse();
        bytes.append(v);
        bytes.append(src_addr);
        v = bcs::to_bytes(&sequence);
        v.reverse();
        bytes.append(v);
        v = bcs::to_bytes(&dst_chain);
        v.reverse();
        bytes.append(v);
        bytes.append(dst_addr);
        bytes.append(payload_hash);
        aptos_hash::keccak256(bytes)
    }

    #[view]
    public fun get_message_status(src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, dst_addr: vector<u8>, payload_hash: vector<u8>): (u128, u128, bool) acquires AttestationState {
        // Returns the enabled receive Transceivers for that chain along with the attestations and the executed flag.
        let integrator_addr = universal_address::from_bytes(dst_addr).to_address();
        let enabled_recv_transceivers = integrator::get_enabled_recv_transceivers(integrator_addr, src_chain);
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        let info = table::borrow(&AttestationState[@router].attestation_infos, message_hash);
        (enabled_recv_transceivers, info.attested_transceivers, info.executed)
    }

    public fun attest_message(transceiver_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, dst_addr: vector<u8>, payload_hash: vector<u8>) acquires AttestationState {
        // MUST check that the Transceiver is an enabled receive Transceiver for the Integrator (`dstAddr`) and chain (`dstChain`).
        let integrator_addr = universal_address::from_bytes(dst_addr).to_address();
        let transceiver_addr = signer::address_of(transceiver_acct);
        let index = integrator::get_transceiver_index(integrator_addr, transceiver_addr);
        let enabled_recv_transceivers = integrator::get_enabled_recv_transceivers(integrator_addr, src_chain);
        let bitmask = 1 << index;
        assert!(enabled_recv_transceivers & bitmask > 0);
        // MUST check that the Transceiver has NOT already attested.
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        let info = table::borrow_mut_with_default(&mut AttestationState[@router].attestation_infos, message_hash, AttestationInfo{executed: false, attested_transceivers: 0});
        assert!(info.attested_transceivers & bitmask == 0);
        // MUST allow a Transceiver to attest after message execution.
        // Calculates the message hash and marks the Transceiver as having attested to the message.
        info.attested_transceivers = info.attested_transceivers | bitmask;
    }

    public fun recvMessage(integrator_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, payload_hash: vector<u8>): (u128, u128) acquires AttestationState {
        // MUST check that at least one Transceiver has attested.
        let integrator_addr = signer::address_of(integrator_acct);
        let dst_addr = universal_address::from_address(integrator_addr).get_bytes();
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        // This borrow will fail if no transceivers have attested.
        let info = table::borrow_mut(&mut AttestationState[@router].attestation_infos, message_hash);
        // MUST revert if already executed.
        assert!(info.executed == false);
        // Marks the message as executed and returns the enabled receive Transceivers for that chain along with the attestations.
        info.executed = true;
        let enabled_recv_transceivers = integrator::get_enabled_recv_transceivers(integrator_addr, src_chain);
        (enabled_recv_transceivers, info.attested_transceivers)
    }

    public fun execMessage(integrator_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, payload_hash: vector<u8>) acquires AttestationState {
        let integrator_addr = signer::address_of(integrator_acct);
        let dst_addr = universal_address::from_address(integrator_addr).get_bytes();
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        // MUST NOT require any Transceivers to have attested
        let info = table::borrow_mut_with_default(&mut AttestationState[@router].attestation_infos, message_hash, AttestationInfo{executed: false, attested_transceivers: 0});
        // MUST revert if already executed.
        assert!(info.executed == false);
        // Marks the message as executed.
        info.executed = true;
    }
}

#[test_only]
module router::router_test {
    use aptos_framework::aptos_hash;
    use router::integrator;
    use router::router;
    use router::universal_address;
    use std::signer;

    const DESTINATION_ADDR: vector<u8> = x"deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    const PAYLOAD_HASH: vector<u8> = x"c3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5";

    #[test(integrator_acct = @0x123)]
    public fun register_test(integrator_acct: &signer) {
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        let admin_addr = integrator::get_admin(integrator_addr);
        assert!(admin_addr.contains(&integrator_addr));
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun register_twice_fails(integrator_acct: &signer) {
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        router::register(integrator_acct, integrator_addr);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun register_with_zero_admin_fails(integrator_acct: &signer) {
        router::register(integrator_acct, @0x0);
    }

    #[test(integrator_acct = @0x123)]
    public fun send_message_test(integrator_acct: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        // prep
        integrator::add_transceiver(integrator_acct, integrator_addr, integrator_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, integrator_addr);
        let sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 0);
        // send a message to chain 1
        let src_addr = universal_address::from_address(integrator_addr);
        let dst_addr = universal_address::from_bytes(DESTINATION_ADDR);
        sequence = router::send_message(integrator_acct, 1, dst_addr, PAYLOAD_HASH);
        assert!(sequence == 0);
        let (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_transceivers) = router::get_outbox_message(integrator_addr, sequence);
        assert!(m_src_addr == src_addr);
        assert!(m_sequence == sequence);
        assert!(m_dst_chain == 1);
        assert!(m_dst_addr == dst_addr);
        assert!(m_payload_hash == PAYLOAD_HASH);
        assert!(m_outstanding_transceivers == 1);
        sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 1);
        // add another transceiver and enable both for chain 5
        integrator::add_transceiver(integrator_acct, integrator_addr, @0x789);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 5, integrator_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 5, @0x789);
        // send another message to chain 5
        sequence = router::send_message(integrator_acct, 5, dst_addr, PAYLOAD_HASH);
        assert!(sequence == 1);
        (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_transceivers) = router::get_outbox_message(integrator_addr, sequence);
        assert!(m_src_addr == src_addr);
        assert!(m_sequence == sequence);
        assert!(m_dst_chain == 5);
        assert!(m_dst_addr == dst_addr);
        assert!(m_payload_hash == PAYLOAD_HASH);
        assert!(m_outstanding_transceivers == 3);
        sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 2);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun send_message_fails_with_bad_hash_length_33(integrator_acct: &signer) {
        router::init_module_test();
        router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), x"c3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5ff");
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun send_message_fails_without_register(integrator_acct: &signer) {
        router::init_module_test();
        router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun send_message_fails_without_enable(integrator_acct: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
    }

    #[test(integrator_addr = @0x123)]
    #[expected_failure]
    public fun get_next_sequence_fails_without_register(integrator_addr: address) {
        integrator::get_next_sequence(integrator_addr);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    public fun pick_up_message_test(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        let tx2_addr = signer::address_of(tx2);
        router::register(integrator_acct, integrator_addr);
        // prep
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx2_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx2_addr);
        // send a message
        let sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        let (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_transceivers) = router::get_outbox_message(integrator_addr, sequence);
        assert!(m_outstanding_transceivers == 3);
        // pickup with 2
        let (t_src_addr, t_sequence, t_dst_chain, t_dst_addr, t_payload_hash, t_outstanding_transceivers) = router::pick_up_message(tx2, integrator_addr, sequence);
        assert!(m_src_addr == t_src_addr);
        assert!(m_sequence == t_sequence);
        assert!(m_dst_chain == t_dst_chain);
        assert!(m_dst_addr == t_dst_addr);
        assert!(m_payload_hash == t_payload_hash);
        assert!(t_outstanding_transceivers == 1);
        let (_, _, _, _, _, new_outstanding_transceivers) = router::get_outbox_message(integrator_addr, sequence);
        assert!(new_outstanding_transceivers == 1);
        // pickup with 1
        let (t_src_addr, t_sequence, t_dst_chain, t_dst_addr, t_payload_hash, t_outstanding_transceivers) = router::pick_up_message(tx1, integrator_addr, sequence);
        assert!(m_src_addr == t_src_addr);
        assert!(m_sequence == t_sequence);
        assert!(m_dst_chain == t_dst_chain);
        assert!(m_dst_addr == t_dst_addr);
        assert!(m_payload_hash == t_payload_hash);
        assert!(t_outstanding_transceivers == 0);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    #[expected_failure]
    public fun pick_up_message_fails_for_disabled_transceiver(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        router::register(integrator_acct, integrator_addr);
        // only register and enable 1
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        // send a message
        let sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        // pickup with 2
        router::pick_up_message(tx2, integrator_addr, sequence);
    }
    
    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    #[expected_failure]
    public fun pick_up_message_twice_fails(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        let tx2_addr = signer::address_of(tx2);
        router::register(integrator_acct, integrator_addr);
        // prep 2 so outbox message still exists after first pickup
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx2_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx2_addr);
        // send a message
        let sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        // pickup with 2 twice
        router::pick_up_message(tx2, integrator_addr, sequence);
        router::pick_up_message(tx2, integrator_addr, sequence);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure]
    public fun get_outbox_message_fails_after_last_pickup(integrator_acct: &signer, tx1: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        router::register(integrator_acct, integrator_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        let sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        router::get_outbox_message(integrator_addr, sequence);
        router::pick_up_message(tx1, integrator_addr, sequence);
        router::get_outbox_message(integrator_addr, sequence);
    }

    #[test]
    public fun compute_message_hash_test() {
        router::compute_message_hash(1, DESTINATION_ADDR, 0, 22, universal_address::from_address(@0x123).get_bytes(), PAYLOAD_HASH);
    }

    #[test]
    public fun compute_message_hash_test_known_hash() {
        let src_addr = universal_address::from_address(@0x123).get_bytes();
        let dst_addr = universal_address::from_address(@0x456).get_bytes();
        let src_chain = 2;
        let dst_chain = 42;
        let sequence = 3;
        let payload_hash = aptos_hash::keccak256(b"hello, world");
        let known_hash = x"f589999616054a74b876390c4eb6e067da272da5cd313a9657d33ec3cab06760";
        let message_hash = router::compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        assert!(known_hash == message_hash);
    }

    #[test]
    #[expected_failure]
    public fun compute_message_hash_fails_with_bad_src_addr_length() {
        router::compute_message_hash(1, x"1234", 0, 22, universal_address::from_address(@0x123).get_bytes(), PAYLOAD_HASH);
    }

    #[test]
    #[expected_failure]
    public fun compute_message_hash_fails_with_bad_dst_addr_length() {
        router::compute_message_hash(1, DESTINATION_ADDR, 0, 22, x"1234", PAYLOAD_HASH);
    }

    #[test]
    #[expected_failure]
    public fun compute_message_hash_fails_with_bad_payload_hash_length() {
        router::compute_message_hash(1, DESTINATION_ADDR, 0, 22, universal_address::from_address(@0x123).get_bytes(), x"1234");
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    public fun attest_message_test(integrator_acct: &signer, tx1: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        router::register(integrator_acct, integrator_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        router::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        let (enabled, attested, executed) = router::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 1);
        assert!(attested == 1);
        assert!(executed == false);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure]
    public fun attest_message_fails_with_disabled_transceiver(integrator_acct: &signer, tx1: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        router::register(integrator_acct, integrator_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::disable_recv_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        router::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure]
    public fun attest_message_twice_fails(integrator_acct: &signer, tx1: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        router::register(integrator_acct, integrator_addr);
        integrator::add_transceiver(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_transceiver(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        router::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        router::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    public fun recv_message_test(integrator_acct: &signer, tx1: &signer) {
        attest_message_test(integrator_acct, tx1);
        router::recvMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
        let integrator_bytes = universal_address::from_address(signer::address_of(integrator_acct)).get_bytes();
        let (enabled, attested, executed) = router::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 1);
        assert!(attested == 1);
        assert!(executed == true);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure]
    public fun recv_message_twice_fails(integrator_acct: &signer, tx1: &signer) {
        recv_message_test(integrator_acct, tx1);
        router::recvMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123)]
    public fun exec_message_test(integrator_acct: &signer) {
        router::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        router::execMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
        let integrator_bytes = universal_address::from_address(signer::address_of(integrator_acct)).get_bytes();
        let (enabled, attested, executed) = router::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 0);
        assert!(attested == 0);
        assert!(executed == true);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure]
    public fun exec_message_twice_fails(integrator_acct: &signer) {
        exec_message_test(integrator_acct);
        router::execMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
    }

}

#[test_only]
module 0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5::integrator_test {
    use std::signer;
    use std::vector;
    use aptos_framework::account;
    use aptos_framework::resource_account;
    use aptos_std::from_bcs;
    use router::integrator;
    use router::router;
    const DEPLOYER: address = @0xcafe;

    fun init_module(resource_account: &signer) {
        // create the resource account that we'll use to send messages
        let resource_signer_cap = resource_account::retrieve_resource_account_cap(resource_account, DEPLOYER);
        let resource_signer = account::create_signer_with_capability(&resource_signer_cap);
        router::register(&resource_signer, DEPLOYER);
    }

    fun set_up_test(origin_account: &signer, resource_account: &signer) {
        // inspired by https://github.com/aptos-labs/aptos-core/blob/2e9d8ee759fcd3f6e831034f05c1656b1c48efc4/aptos-move/move-examples/mint_nft/sources/minting.move#L207C27-L220
        // create the origin_account so it has an authentication key for the resource account setup
        account::create_account_for_test(signer::address_of(origin_account));
        // create a resource account from the origin account, mocking the module publishing process
        resource_account::create_resource_account(origin_account, vector::empty<u8>(), vector::empty<u8>());
        init_module(resource_account);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun initial_admin_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        // ensure the admin was set to the addess defined in the init
        let resource_addr = signer::address_of(resource_account);
        let admin_addr = integrator::get_admin(resource_addr);
        assert!(admin_addr.contains(&DEPLOYER));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun update_admin_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let new_admin: address = @0xdeadbeef;
        let resource_addr = signer::address_of(resource_account);
        integrator::update_admin(origin_account, resource_addr, new_admin);
        assert!(integrator::get_admin(resource_addr).contains(&new_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun update_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::update_admin(wrong_admin, resource_addr, signer::address_of(wrong_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun update_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::update_admin(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun update_admin_fails_with_zero_address(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::update_admin(origin_account, resource_addr, @0x0);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun transfer_admin_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let new_admin: address = @0xdeadbeef;
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, new_admin);
        assert!(integrator::get_admin(resource_addr).contains(&signer::address_of(origin_account)));
        assert!(integrator::get_pending_admin(resource_addr).contains(&new_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun transfer_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(wrong_admin, resource_addr, signer::address_of(wrong_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun transfer_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun transfer_admin_fails_with_zero_address(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, @0x0);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun claim_admin_test_cancel(origin_account: &signer, resource_account: &signer) {
        transfer_admin_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(origin_account, resource_addr);
        assert!(integrator::get_admin(resource_addr).contains(&signer::address_of(origin_account)));
        assert!(integrator::get_pending_admin(resource_addr).is_none());
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, new_admin = @0xdeadbeef)]
    fun claim_admin_test_complete(origin_account: &signer, resource_account: &signer, new_admin: &signer) {
        transfer_admin_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(new_admin, resource_addr);
        assert!(integrator::get_admin(resource_addr).contains(&signer::address_of(new_admin)));
        assert!(integrator::get_pending_admin(resource_addr).is_none());
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun claim_admin_fails_without_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(origin_account, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xbeef5)]
    #[expected_failure]
    fun claim_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        transfer_admin_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(wrong_admin, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun discard_admin_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::discard_admin(origin_account, resource_addr);
        assert!(integrator::get_admin(resource_addr).is_none());
        assert!(integrator::get_pending_admin(resource_addr).is_none());
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun discard_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::discard_admin(wrong_admin, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun discard_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::discard_admin(origin_account, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun add_transceiver_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        let transceivers = integrator::get_transceivers(resource_addr);
        assert!(vector::is_empty(&transceivers));
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        transceivers = integrator::get_transceivers(resource_addr);
        assert!(vector::length(&transceivers) == 1);
        assert!(transceivers[0] == resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun add_transceiver_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(wrong_admin, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun add_transceiver_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun add_transceiver_fails_with_duplicate(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun add_transceiver_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let transceiver_addr = from_bcs::to_address(v);
            integrator::add_transceiver(origin_account, integrator_addr, transceiver_addr);
            assert!(integrator::get_transceiver_index(integrator_addr, transceiver_addr) == i);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun add_transceiver_129_test(origin_account: &signer, resource_account: &signer) {
        add_transceiver_128_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        let v = x"0000000000000000000000000000000000000000000000000000000000FF0100";
        let transceiver_addr = from_bcs::to_address(v);
        integrator::add_transceiver(origin_account, integrator_addr, transceiver_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_send_transceiver_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        let expected_bitmap = 0;
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let transceiver_addr = from_bcs::to_address(v);
            integrator::add_transceiver(origin_account, integrator_addr, transceiver_addr);
            integrator::enable_send_transceiver(origin_account, integrator_addr, 1, transceiver_addr);
            let bitmap = integrator::get_enabled_send_transceivers(integrator_addr, 1);
            expected_bitmap = expected_bitmap + (1 << i);
            assert!(bitmap == expected_bitmap);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun enable_send_transceiver_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::enable_send_transceiver(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun enable_send_transceiver_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::enable_send_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun enable_send_transceiver_fails_with_enabled_transceiver(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        integrator::enable_send_transceiver(origin_account, resource_addr, 1, resource_addr);
        integrator::enable_send_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun disable_send_transceiver_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::disable_send_transceiver(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun disable_send_transceiver_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::disable_send_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun disable_send_transceiver_fails_with_disabled_transceiver(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        integrator::disable_send_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_and_disable_send_transceiver_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let transceiver_addr = from_bcs::to_address(v);
            integrator::add_transceiver(origin_account, integrator_addr, transceiver_addr);
            integrator::enable_send_transceiver(origin_account, integrator_addr, 1, transceiver_addr);
            let bitmap = integrator::get_enabled_send_transceivers(integrator_addr, 1);
            assert!(bitmap == (1 << i));
            integrator::disable_send_transceiver(origin_account, integrator_addr, 1, transceiver_addr);
            let bitmap = integrator::get_enabled_send_transceivers(integrator_addr, 1);
            assert!(bitmap == 0);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun enable_recv_transceiver_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::enable_recv_transceiver(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun enable_recv_transceiver_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::enable_recv_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun enable_recv_transceiver_fails_with_enabled_transceiver(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        integrator::enable_recv_transceiver(origin_account, resource_addr, 1, resource_addr);
        integrator::enable_recv_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure]
    fun disable_recv_transceiver_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::disable_recv_transceiver(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun disable_recv_transceiver_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::disable_recv_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure]
    fun disable_recv_transceiver_fails_with_disabled_transceiver(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_transceiver(origin_account, resource_addr, resource_addr);
        integrator::disable_recv_transceiver(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_and_disable_recv_transceiver_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let transceiver_addr = from_bcs::to_address(v);
            integrator::add_transceiver(origin_account, integrator_addr, transceiver_addr);
            integrator::enable_recv_transceiver(origin_account, integrator_addr, 1, transceiver_addr);
            let bitmap = integrator::get_enabled_recv_transceivers(integrator_addr, 1);
            assert!(bitmap == (1 << i));
            integrator::disable_recv_transceiver(origin_account, integrator_addr, 1, transceiver_addr);
            let bitmap = integrator::get_enabled_recv_transceivers(integrator_addr, 1);
            assert!(bitmap == 0);
        }
    }
}
