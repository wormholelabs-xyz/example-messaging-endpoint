module endpoint::endpoint {
    use aptos_framework::aptos_hash;
    use endpoint::bitmap;
    use endpoint::integrator;
    use endpoint::universal_address::{Self, UniversalAddress};
    use std::bcs;
    use std::signer;
    use std::table::{Self, Table};
    use std::vector;

    const E_INVALID_PAYLOAD_HASH_LENGTH: u64 = 0;
    const E_ADAPTER_NOT_ENABLED: u64 = 1;
    const E_INVALID_SOURCE_ADDRESS_LENGTH: u64 = 2;
    const E_INVALID_DESTINATION_ADDRESS_LENGTH: u64 = 3;
    const E_ALREADY_EXECUTED: u64 = 4;

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
        /// The bitmap of send-enabled adapters for this destination chain that have not picked up the message
        outstanding_adapters: u128
    }

    struct OutboxState has key {
        outbox_messages: Table<OutboxMessageKey, OutboxMessage>
    }

    struct AttestationInfo has copy, drop, store {
        /// Replay protection flag
        executed: bool,
        /// The bitmap of receive-enabled adapters for this source chain that have attested to the message
        attested_adapters: u128
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
        let signer_cap = account::create_test_signer_cap(@endpoint);
        let signer = account::create_signer_with_capability(&signer_cap);
        // then call the initialiser
        init_module(&signer)
    }

    public fun register(integrator_acct: &signer, admin_addr: address) {
        integrator::new_integrator(integrator_acct, admin_addr);
    }

    public fun send_message(integrator_acct: &signer, dst_chain: u16, dst_addr: UniversalAddress, payload_hash: vector<u8>): u64 acquires OutboxState {
        assert!(vector::length(&payload_hash) == 32, E_INVALID_PAYLOAD_HASH_LENGTH);
        let integrator_addr = signer::address_of(integrator_acct);
        // MUST have at least one enabled send Adapter for `dstChain`.
        let outstanding_adapters = integrator::get_enabled_send_adapters(integrator_addr, dst_chain);
        assert!(outstanding_adapters != 0, E_ADAPTER_NOT_ENABLED);
        // Increments the Integrator's sequence, creates and stores the outbox item.
        // MUST set the current enabled Send Adapters as the Outstanding Adapters for that message.
        let src_addr = universal_address::from_address(integrator_addr);
        let sequence = integrator::use_sequence(integrator_acct);
        table::add(&mut OutboxState[@endpoint].outbox_messages, OutboxMessageKey{integrator_addr, sequence}, OutboxMessage {
            src_addr, sequence, dst_chain, dst_addr, payload_hash, outstanding_adapters
        });
        sequence
    }

    #[view]
    public fun get_outbox_message(integrator_addr: address, sequence: u64): (UniversalAddress, u64, u16, UniversalAddress, vector<u8>, u128) acquires OutboxState {
        let message = table::borrow(&OutboxState[@endpoint].outbox_messages, OutboxMessageKey{integrator_addr, sequence});
        (message.src_addr, message.sequence, message.dst_chain, message.dst_addr, message.payload_hash, message.outstanding_adapters)
    }

    public fun pick_up_message(adapter_acct: &signer, integrator_addr: address, sequence: u64): (UniversalAddress, u64, u16, UniversalAddress, vector<u8>, u128) acquires OutboxState {
        let key = OutboxMessageKey{integrator_addr, sequence};
        let message = table::borrow(&OutboxState[@endpoint].outbox_messages, key);
        // MUST check that the Adapter is an enabled send Adapter for the Integrator (`srcAddr`) and chain (`dstChain`).
        // Since the enabled adapters are copied to the outstanding adapters field when the outbox message is generated,
        // this just needs the corresponding index to check against.
        let adapter_addr = signer::address_of(adapter_acct);
        let index = integrator::get_adapter_index(integrator_addr, adapter_addr);
        // MUST check that the Adapter has NOT already picked up the message.
        // Marks the Adapter as having picked up the message.
        let new_outstanding_adapters = bitmap::disable(message.outstanding_adapters, index);
        // In order to reduce integrator / user costs, upon the last enabled sending Adapter's pickup, any outgoing message state MUST be cleared.
        if (new_outstanding_adapters == 0) {
            let message = table::remove(&mut OutboxState[@endpoint].outbox_messages, key);
            (message.src_addr, message.sequence, message.dst_chain, message.dst_addr, message.payload_hash, new_outstanding_adapters)
        } else {
            let mut_message = table::borrow_mut(&mut OutboxState[@endpoint].outbox_messages, key);
            mut_message.outstanding_adapters = new_outstanding_adapters;
            (mut_message.src_addr, mut_message.sequence, mut_message.dst_chain, mut_message.dst_addr, mut_message.payload_hash, new_outstanding_adapters)
        }
    }

    #[view]
    public fun compute_message_hash(src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, dst_addr: vector<u8>, payload_hash: vector<u8>): vector<u8> {
        assert!(src_addr.length() == 32, E_INVALID_SOURCE_ADDRESS_LENGTH);
        assert!(dst_addr.length() == 32, E_INVALID_DESTINATION_ADDRESS_LENGTH);
        assert!(payload_hash.length() == 32, E_INVALID_PAYLOAD_HASH_LENGTH);
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
        // Returns the enabled receive Adapters for that chain along with the attestations and the executed flag.
        let integrator_addr = universal_address::from_bytes(dst_addr).to_address();
        let enabled_recv_adapters = integrator::get_enabled_recv_adapters(integrator_addr, src_chain);
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        let info = table::borrow(&AttestationState[@endpoint].attestation_infos, message_hash);
        (enabled_recv_adapters, info.attested_adapters, info.executed)
    }

    public fun attest_message(adapter_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, dst_addr: vector<u8>, payload_hash: vector<u8>) acquires AttestationState {
        // MUST check that the Adapter is an enabled receive Adapter for the Integrator (`dstAddr`) and chain (`dstChain`).
        let integrator_addr = universal_address::from_bytes(dst_addr).to_address();
        let adapter_addr = signer::address_of(adapter_acct);
        let index = integrator::get_adapter_index(integrator_addr, adapter_addr);
        let enabled_recv_adapters = integrator::get_enabled_recv_adapters(integrator_addr, src_chain);
        assert!(bitmap::get(enabled_recv_adapters, index), E_ADAPTER_NOT_ENABLED);
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        let info = table::borrow_mut_with_default(&mut AttestationState[@endpoint].attestation_infos, message_hash, AttestationInfo{executed: false, attested_adapters: 0});
        // MUST check that the Adapter has NOT already attested.
        // MUST allow an Adapter to attest after message execution.
        // Calculates the message hash and marks the Adapter as having attested to the message.
        info.attested_adapters = bitmap::enable(info.attested_adapters, index);
    }

    public fun recvMessage(integrator_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, payload_hash: vector<u8>): (u128, u128) acquires AttestationState {
        // MUST check that at least one Adapter has attested.
        let integrator_addr = signer::address_of(integrator_acct);
        let dst_addr = universal_address::from_address(integrator_addr).get_bytes();
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        // This borrow will fail if no adapters have attested.
        let info = table::borrow_mut(&mut AttestationState[@endpoint].attestation_infos, message_hash);
        // MUST revert if already executed.
        assert!(info.executed == false, E_ALREADY_EXECUTED);
        // Marks the message as executed and returns the enabled receive Adapters for that chain along with the attestations.
        info.executed = true;
        let enabled_recv_adapters = integrator::get_enabled_recv_adapters(integrator_addr, src_chain);
        (enabled_recv_adapters, info.attested_adapters)
    }

    public fun execMessage(integrator_acct: &signer, src_chain: u16, src_addr: vector<u8>, sequence: u64, dst_chain: u16, payload_hash: vector<u8>) acquires AttestationState {
        let integrator_addr = signer::address_of(integrator_acct);
        let dst_addr = universal_address::from_address(integrator_addr).get_bytes();
        let message_hash = compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        // MUST NOT require any Adapters to have attested
        let info = table::borrow_mut_with_default(&mut AttestationState[@endpoint].attestation_infos, message_hash, AttestationInfo{executed: false, attested_adapters: 0});
        // MUST revert if already executed.
        assert!(info.executed == false, E_ALREADY_EXECUTED);
        // Marks the message as executed.
        info.executed = true;
    }
}

#[test_only]
module endpoint::endpoint_test {
    use aptos_framework::aptos_hash;
    use aptos_framework::table;
    use endpoint::bitmap;
    use endpoint::integrator;
    use endpoint::endpoint::{Self, E_INVALID_PAYLOAD_HASH_LENGTH, E_ADAPTER_NOT_ENABLED, E_INVALID_SOURCE_ADDRESS_LENGTH, E_INVALID_DESTINATION_ADDRESS_LENGTH, E_ALREADY_EXECUTED};
    use endpoint::universal_address;
    use std::signer;

    const DESTINATION_ADDR: vector<u8> = x"deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    const PAYLOAD_HASH: vector<u8> = x"c3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5";

    #[test(integrator_acct = @0x123)]
    public fun register_test(integrator_acct: &signer) {
        let integrator_addr = signer::address_of(integrator_acct);
        endpoint::register(integrator_acct, integrator_addr);
        let admin_addr = integrator::get_admin(integrator_addr);
        assert!(admin_addr.contains(&integrator_addr));
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(abort_code = integrator::E_ALREADY_REGISTERED, location = integrator)]
    public fun register_twice_fails(integrator_acct: &signer) {
        let integrator_addr = signer::address_of(integrator_acct);
        endpoint::register(integrator_acct, integrator_addr);
        endpoint::register(integrator_acct, integrator_addr);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(abort_code = integrator::E_INVALID_ADMIN, location = integrator)]
    public fun register_with_zero_admin_fails(integrator_acct: &signer) {
        endpoint::register(integrator_acct, @0x0);
    }

    #[test(integrator_acct = @0x123)]
    public fun send_message_test(integrator_acct: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        endpoint::register(integrator_acct, integrator_addr);
        // prep
        integrator::add_adapter(integrator_acct, integrator_addr, integrator_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, integrator_addr);
        let sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 0);
        // send a message to chain 1
        let src_addr = universal_address::from_address(integrator_addr);
        let dst_addr = universal_address::from_bytes(DESTINATION_ADDR);
        sequence = endpoint::send_message(integrator_acct, 1, dst_addr, PAYLOAD_HASH);
        assert!(sequence == 0);
        let (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_adapters) = endpoint::get_outbox_message(integrator_addr, sequence);
        assert!(m_src_addr == src_addr);
        assert!(m_sequence == sequence);
        assert!(m_dst_chain == 1);
        assert!(m_dst_addr == dst_addr);
        assert!(m_payload_hash == PAYLOAD_HASH);
        assert!(m_outstanding_adapters == 1);
        sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 1);
        // add another adapter and enable both for chain 5
        integrator::add_adapter(integrator_acct, integrator_addr, @0x789);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 5, integrator_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 5, @0x789);
        // send another message to chain 5
        sequence = endpoint::send_message(integrator_acct, 5, dst_addr, PAYLOAD_HASH);
        assert!(sequence == 1);
        (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_adapters) = endpoint::get_outbox_message(integrator_addr, sequence);
        assert!(m_src_addr == src_addr);
        assert!(m_sequence == sequence);
        assert!(m_dst_chain == 5);
        assert!(m_dst_addr == dst_addr);
        assert!(m_payload_hash == PAYLOAD_HASH);
        assert!(m_outstanding_adapters == 3);
        sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 2);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(abort_code = E_INVALID_PAYLOAD_HASH_LENGTH, location = endpoint)]
    public fun send_message_fails_with_bad_hash_length_33(integrator_acct: &signer) {
        endpoint::init_module_test();
        endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), x"c3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5ff");
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(major_status = 4008, location = integrator)]
    public fun send_message_fails_without_register(integrator_acct: &signer) {
        endpoint::init_module_test();
        endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(abort_code = E_ADAPTER_NOT_ENABLED, location = endpoint)]
    public fun send_message_fails_without_enable(integrator_acct: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        endpoint::register(integrator_acct, integrator_addr);
        endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
    }

    #[test(integrator_addr = @0x123)]
    #[expected_failure(major_status = 4008, location = integrator)]
    public fun get_next_sequence_fails_without_register(integrator_addr: address) {
        integrator::get_next_sequence(integrator_addr);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    public fun pick_up_message_test(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        let tx2_addr = signer::address_of(tx2);
        endpoint::register(integrator_acct, integrator_addr);
        // prep
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx2_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx2_addr);
        // send a message
        let sequence = endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        let (m_src_addr, m_sequence, m_dst_chain, m_dst_addr, m_payload_hash, m_outstanding_adapters) = endpoint::get_outbox_message(integrator_addr, sequence);
        assert!(m_outstanding_adapters == 3);
        // pickup with 2
        let (t_src_addr, t_sequence, t_dst_chain, t_dst_addr, t_payload_hash, t_outstanding_adapters) = endpoint::pick_up_message(tx2, integrator_addr, sequence);
        assert!(m_src_addr == t_src_addr);
        assert!(m_sequence == t_sequence);
        assert!(m_dst_chain == t_dst_chain);
        assert!(m_dst_addr == t_dst_addr);
        assert!(m_payload_hash == t_payload_hash);
        assert!(t_outstanding_adapters == 1);
        let (_, _, _, _, _, new_outstanding_adapters) = endpoint::get_outbox_message(integrator_addr, sequence);
        assert!(new_outstanding_adapters == 1);
        // pickup with 1
        let (t_src_addr, t_sequence, t_dst_chain, t_dst_addr, t_payload_hash, t_outstanding_adapters) = endpoint::pick_up_message(tx1, integrator_addr, sequence);
        assert!(m_src_addr == t_src_addr);
        assert!(m_sequence == t_sequence);
        assert!(m_dst_chain == t_dst_chain);
        assert!(m_dst_addr == t_dst_addr);
        assert!(m_payload_hash == t_payload_hash);
        assert!(t_outstanding_adapters == 0);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    #[expected_failure(abort_code = 25863, location = table)]
    public fun pick_up_message_fails_for_disabled_adapter(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        endpoint::register(integrator_acct, integrator_addr);
        // only register and enable 1
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        // send a message
        let sequence = endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        // pickup with 2
        endpoint::pick_up_message(tx2, integrator_addr, sequence);
    }
    
    #[test(integrator_acct = @0x123, tx1 = @0x456, tx2 = @0x789)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_DISABLED, location = bitmap)]
    public fun pick_up_message_twice_fails(integrator_acct: &signer, tx1: &signer, tx2: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        let tx2_addr = signer::address_of(tx2);
        endpoint::register(integrator_acct, integrator_addr);
        // prep 2 so outbox message still exists after first pickup
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx2_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx2_addr);
        // send a message
        let sequence = endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        // pickup with 2 twice
        endpoint::pick_up_message(tx2, integrator_addr, sequence);
        endpoint::pick_up_message(tx2, integrator_addr, sequence);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure(abort_code = 25863, location = table)]
    public fun get_outbox_message_fails_after_last_pickup(integrator_acct: &signer, tx1: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        endpoint::register(integrator_acct, integrator_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_send_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        let sequence = endpoint::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        endpoint::get_outbox_message(integrator_addr, sequence);
        endpoint::pick_up_message(tx1, integrator_addr, sequence);
        endpoint::get_outbox_message(integrator_addr, sequence);
    }

    #[test]
    public fun compute_message_hash_test() {
        endpoint::compute_message_hash(1, DESTINATION_ADDR, 0, 22, universal_address::from_address(@0x123).get_bytes(), PAYLOAD_HASH);
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
        let message_hash = endpoint::compute_message_hash(src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash);
        assert!(known_hash == message_hash);
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_SOURCE_ADDRESS_LENGTH, location = endpoint)]
    public fun compute_message_hash_fails_with_bad_src_addr_length() {
        endpoint::compute_message_hash(1, x"1234", 0, 22, universal_address::from_address(@0x123).get_bytes(), PAYLOAD_HASH);
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_DESTINATION_ADDRESS_LENGTH, location = endpoint)]
    public fun compute_message_hash_fails_with_bad_dst_addr_length() {
        endpoint::compute_message_hash(1, DESTINATION_ADDR, 0, 22, x"1234", PAYLOAD_HASH);
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_PAYLOAD_HASH_LENGTH, location = endpoint)]
    public fun compute_message_hash_fails_with_bad_payload_hash_length() {
        endpoint::compute_message_hash(1, DESTINATION_ADDR, 0, 22, universal_address::from_address(@0x123).get_bytes(), x"1234");
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    public fun attest_message_test(integrator_acct: &signer, tx1: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        endpoint::register(integrator_acct, integrator_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        endpoint::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        let (enabled, attested, executed) = endpoint::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 1);
        assert!(attested == 1);
        assert!(executed == false);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure(abort_code = E_ADAPTER_NOT_ENABLED, location = endpoint)]
    public fun attest_message_fails_with_disabled_adapter(integrator_acct: &signer, tx1: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        endpoint::register(integrator_acct, integrator_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        integrator::disable_recv_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        endpoint::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_ENABLED, location = bitmap)]
    public fun attest_message_twice_fails(integrator_acct: &signer, tx1: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        let tx1_addr = signer::address_of(tx1);
        endpoint::register(integrator_acct, integrator_addr);
        integrator::add_adapter(integrator_acct, integrator_addr, tx1_addr);
        integrator::enable_recv_adapter(integrator_acct, integrator_addr, 1, tx1_addr);
        let integrator_bytes = universal_address::from_address(integrator_addr).get_bytes();
        endpoint::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        endpoint::attest_message(tx1, 1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    public fun recv_message_test(integrator_acct: &signer, tx1: &signer) {
        attest_message_test(integrator_acct, tx1);
        endpoint::recvMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
        let integrator_bytes = universal_address::from_address(signer::address_of(integrator_acct)).get_bytes();
        let (enabled, attested, executed) = endpoint::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 1);
        assert!(attested == 1);
        assert!(executed == true);
    }

    #[test(integrator_acct = @0x123, tx1 = @0x456)]
    #[expected_failure(abort_code = E_ALREADY_EXECUTED, location = endpoint)]
    public fun recv_message_twice_fails(integrator_acct: &signer, tx1: &signer) {
        recv_message_test(integrator_acct, tx1);
        endpoint::recvMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
    }

    #[test(integrator_acct = @0x123)]
    public fun exec_message_test(integrator_acct: &signer) {
        endpoint::init_module_test();
        let integrator_addr = signer::address_of(integrator_acct);
        endpoint::register(integrator_acct, integrator_addr);
        endpoint::execMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
        let integrator_bytes = universal_address::from_address(signer::address_of(integrator_acct)).get_bytes();
        let (enabled, attested, executed) = endpoint::get_message_status(1, DESTINATION_ADDR, 0, 22, integrator_bytes, PAYLOAD_HASH);
        assert!(enabled == 0);
        assert!(attested == 0);
        assert!(executed == true);
    }

    #[test(integrator_acct = @0x123)]
    #[expected_failure(abort_code = E_ALREADY_EXECUTED, location = endpoint)]
    public fun exec_message_twice_fails(integrator_acct: &signer) {
        exec_message_test(integrator_acct);
        endpoint::execMessage(integrator_acct, 1, DESTINATION_ADDR, 0, 22, PAYLOAD_HASH);
    }

}

#[test_only]
module 0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5::integrator_test {
    use std::signer;
    use std::vector;
    use aptos_framework::account;
    use aptos_framework::resource_account;
    use aptos_std::from_bcs;
    use endpoint::bitmap;
    use endpoint::integrator;
    use endpoint::endpoint;
    const DEPLOYER: address = @0xcafe;

    fun init_module(resource_account: &signer) {
        // create the resource account that we'll use to send messages
        let resource_signer_cap = resource_account::retrieve_resource_account_cap(resource_account, DEPLOYER);
        let resource_signer = account::create_signer_with_capability(&resource_signer_cap);
        endpoint::register(&resource_signer, DEPLOYER);
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
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun update_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::update_admin(wrong_admin, resource_addr, signer::address_of(wrong_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun update_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::update_admin(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_INVALID_ADMIN, location = integrator)]
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
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun transfer_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(wrong_admin, resource_addr, signer::address_of(wrong_admin));
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun transfer_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_INVALID_ADMIN, location = integrator)]
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
    #[expected_failure(abort_code = integrator::E_NO_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun claim_admin_fails_without_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(origin_account, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xbeef5)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
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
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun discard_admin_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::discard_admin(wrong_admin, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun discard_admin_fails_with_pending_transfer(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::discard_admin(origin_account, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun add_adapter_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        let adapters = integrator::get_adapters(resource_addr);
        assert!(vector::is_empty(&adapters));
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        adapters = integrator::get_adapters(resource_addr);
        assert!(vector::length(&adapters) == 1);
        assert!(adapters[0] == resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun add_adapter_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(wrong_admin, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun add_adapter_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ALREADY_REGISTERED, location = integrator)]
    fun add_adapter_fails_with_duplicate(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun add_adapter_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let adapter_addr = from_bcs::to_address(v);
            integrator::add_adapter(origin_account, integrator_addr, adapter_addr);
            assert!(integrator::get_adapter_index(integrator_addr, adapter_addr) == i);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_MAX_ADAPTERS_REACHED, location = integrator)]
    fun add_adapter_129_test(origin_account: &signer, resource_account: &signer) {
        add_adapter_128_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        let v = x"0000000000000000000000000000000000000000000000000000000000FF0100";
        let adapter_addr = from_bcs::to_address(v);
        integrator::add_adapter(origin_account, integrator_addr, adapter_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_send_adapter_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        let expected_bitmap = 0;
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let adapter_addr = from_bcs::to_address(v);
            integrator::add_adapter(origin_account, integrator_addr, adapter_addr);
            integrator::enable_send_adapter(origin_account, integrator_addr, 1, adapter_addr);
            let bitmap = integrator::get_enabled_send_adapters(integrator_addr, 1);
            expected_bitmap = expected_bitmap + (1 << i);
            assert!(bitmap == expected_bitmap);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun enable_send_adapter_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::enable_send_adapter(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun enable_send_adapter_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::enable_send_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_ENABLED, location = bitmap)]
    fun enable_send_adapter_fails_with_enabled_adapter(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        integrator::enable_send_adapter(origin_account, resource_addr, 1, resource_addr);
        integrator::enable_send_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun disable_send_adapter_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::disable_send_adapter(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun disable_send_adapter_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::disable_send_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_DISABLED, location = bitmap)]
    fun disable_send_adapter_fails_with_disabled_adapter(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        integrator::disable_send_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_and_disable_send_adapter_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let adapter_addr = from_bcs::to_address(v);
            integrator::add_adapter(origin_account, integrator_addr, adapter_addr);
            integrator::enable_send_adapter(origin_account, integrator_addr, 1, adapter_addr);
            let bitmap = integrator::get_enabled_send_adapters(integrator_addr, 1);
            assert!(bitmap == (1 << i));
            integrator::disable_send_adapter(origin_account, integrator_addr, 1, adapter_addr);
            let bitmap = integrator::get_enabled_send_adapters(integrator_addr, 1);
            assert!(bitmap == 0);
        }
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun enable_recv_adapter_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::enable_recv_adapter(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun enable_recv_adapter_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::enable_recv_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_ENABLED, location = bitmap)]
    fun enable_recv_adapter_fails_with_enabled_adapter(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        integrator::enable_recv_adapter(origin_account, resource_addr, 1, resource_addr);
        integrator::enable_recv_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, wrong_admin = @0xdeadbeef)]
    #[expected_failure(abort_code = integrator::E_NOT_AUTHORIZED, location = integrator)]
    fun disable_recv_adapter_fails_with_wrong_admin(origin_account: &signer, resource_account: &signer, wrong_admin: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::disable_recv_adapter(wrong_admin, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = integrator::E_ADMIN_TRANSFER_IN_PROGRESS, location = integrator)]
    fun disable_recv_adapter_fails_with_pending_admin(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::transfer_admin(origin_account, resource_addr, resource_addr);
        integrator::disable_recv_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    #[expected_failure(abort_code = bitmap::E_ALREADY_DISABLED, location = bitmap)]
    fun disable_recv_adapter_fails_with_disabled_adapter(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::add_adapter(origin_account, resource_addr, resource_addr);
        integrator::disable_recv_adapter(origin_account, resource_addr, 1, resource_addr);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun enable_and_disable_recv_adapter_128_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let integrator_addr = signer::address_of(resource_account);
        for (i in 0..128) {
            let v = x"0000000000000000000000000000000000000000000000000000000000FF00";
            vector::push_back(&mut v, i);
            let adapter_addr = from_bcs::to_address(v);
            integrator::add_adapter(origin_account, integrator_addr, adapter_addr);
            integrator::enable_recv_adapter(origin_account, integrator_addr, 1, adapter_addr);
            let bitmap = integrator::get_enabled_recv_adapters(integrator_addr, 1);
            assert!(bitmap == (1 << i));
            integrator::disable_recv_adapter(origin_account, integrator_addr, 1, adapter_addr);
            let bitmap = integrator::get_enabled_recv_adapters(integrator_addr, 1);
            assert!(bitmap == 0);
        }
    }
}
