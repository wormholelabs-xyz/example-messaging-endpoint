module router::router {
    use router::integrator;
    use router::universal_address::{Self, UniversalAddress};
    use std::signer;
    use std::table::{Self, Table};
    use std::vector;

    struct OutboxMessageKey has copy, drop {
        src_addr: UniversalAddress,
        sequence: u64
    }

    struct OutboxMessage has store {
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

    struct RouterState has key {
        outbox_messages: Table<OutboxMessageKey, OutboxMessage>
    }

    fun init_module(resource_account: &signer) {
        move_to(resource_account, RouterState {
            outbox_messages: table::new<OutboxMessageKey, OutboxMessage>(),
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

    public fun send_message(integrator_acct: &signer, dst_chain: u16, dst_addr: UniversalAddress, payload_hash: vector<u8>): u64 acquires RouterState {
        assert!(vector::length(&payload_hash) == 32);
        let integrator_addr = signer::address_of(integrator_acct);
        // MUST have at least one enabled send Transceiver for `dstChain`.
        let outstanding_transceivers = integrator::get_enabled_send_transceivers(integrator_addr, dst_chain);
        assert!(outstanding_transceivers != 0);
        // Increments the Integrator's sequence, creates and stores the outbox item.
        // MUST set the current enabled Send Transceivers as the Outstanding Transceivers for that message.
        let src_addr = universal_address::from_address(integrator_addr);
        let sequence = integrator::use_sequence(integrator_acct);
        table::add(&mut RouterState[@router].outbox_messages, OutboxMessageKey{src_addr, sequence}, OutboxMessage {
            src_addr, sequence, dst_chain, dst_addr, payload_hash, outstanding_transceivers
        });
        sequence
    }

}

#[test_only]
module router::router_test {
    use router::integrator;
    use router::router;
    use router::universal_address;
    use std::signer;

    const DESTINATION_ADDR: vector<u8> = x"";
    const PAYLOAD_HASH: vector<u8> = x"c3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5";

    #[test(integrator_acct = @0x123)]
    public fun register_test(integrator_acct: &signer) {
        let integrator_addr = signer::address_of(integrator_acct);
        router::register(integrator_acct, integrator_addr);
        let admin_addr = integrator::get_admin(integrator_addr);
        assert!(admin_addr == integrator_addr);
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
        integrator::add_transceiver(integrator_acct, integrator_addr, integrator_addr);
        integrator::enable_send_transceiver(integrator_acct, integrator_addr, 1, integrator_addr);
        let sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 0);
        sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        assert!(sequence == 0);
        sequence = integrator::get_next_sequence(integrator_addr);
        assert!(sequence == 1);
        sequence = router::send_message(integrator_acct, 1, universal_address::from_bytes(DESTINATION_ADDR), PAYLOAD_HASH);
        assert!(sequence == 1);
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
        assert!(admin_addr == DEPLOYER);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5)]
    fun update_admin_test(origin_account: &signer, resource_account: &signer) {
        set_up_test(origin_account, resource_account);
        let new_admin: address = @0xdeadbeef;
        let resource_addr = signer::address_of(resource_account);
        integrator::update_admin(origin_account, resource_addr, new_admin);
        assert!(integrator::get_admin(resource_addr) == new_admin);
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
        assert!(integrator::get_admin(resource_addr) == signer::address_of(origin_account));
        assert!(integrator::get_pending_admin(resource_addr) == new_admin);
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
        assert!(integrator::get_admin(resource_addr) == signer::address_of(origin_account));
        assert!(integrator::get_pending_admin(resource_addr) == @0x0);
    }

    #[test(origin_account = @0xcafe, resource_account = @0xc3bb8488ab1a5815a9d543d7e41b0e0df46a7396f89b22821f07a4362f75ddc5, new_admin = @0xdeadbeef)]
    fun claim_admin_test_complete(origin_account: &signer, resource_account: &signer, new_admin: &signer) {
        transfer_admin_test(origin_account, resource_account);
        let resource_addr = signer::address_of(resource_account);
        integrator::claim_admin(new_admin, resource_addr);
        assert!(integrator::get_admin(resource_addr) == signer::address_of(new_admin));
        assert!(integrator::get_pending_admin(resource_addr) == @0x0);
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
        assert!(integrator::get_admin(resource_addr) == @0x0);
        assert!(integrator::get_pending_admin(resource_addr) == @0x0);
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
