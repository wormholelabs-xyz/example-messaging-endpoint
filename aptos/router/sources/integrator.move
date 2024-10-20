module router::integrator {
    use std::signer;
    use std::table::{Self, Table};
    use std::vector;

    const MAX_U128: u128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    const MAX_TRANSCEIVERS: u64 = 128;

    struct IntegratorCapability has key, store {
        /// Sequence number of the next message
        sequence: u64
    }

    struct AdminConfig has key, store {
        // The address of the admin
        addr: address,
        // The address of the pending admin
        pending_addr: address
    }

    struct TransceiversStore has key, store {
        transceivers: vector<address>
    }

    struct TransceiverToIndexStore has key, store {
        transceiver_to_index: Table<address, u8>
    }

    struct SendTransceiverStore has key, store {
        per_chain_transceiver_bitmap: Table<u16, u128>
    }

    struct RecvTransceiverStore has key, store {
        per_chain_transceiver_bitmap: Table<u16, u128>
    }

    package fun new_integrator(integrator_acct: &signer, admin_addr: address) {
        let integrator_addr = signer::address_of(integrator_acct);
        // MUST check that the caller (Integrator) is not already registered.
        assert!(!exists<IntegratorCapability>(integrator_addr));
        // If possible, MUST check that the admin is potentially valid / non-null.
        assert!(admin_addr != @0x0);
        // Initializes their registration and sets the initial admin.
        move_to(integrator_acct, IntegratorCapability { sequence: 0 });
        move_to(integrator_acct, AdminConfig{addr: admin_addr, pending_addr: @0x0});
        move_to(integrator_acct, TransceiversStore{transceivers: vector::empty()});
        move_to(integrator_acct, TransceiverToIndexStore{transceiver_to_index: table::new<address, u8>()});
        move_to(integrator_acct, SendTransceiverStore{per_chain_transceiver_bitmap: table::new<u16, u128>()});
        move_to(integrator_acct, RecvTransceiverStore{per_chain_transceiver_bitmap: table::new<u16, u128>()});
    }

    #[view]
    public fun get_admin(integrator_addr: address): address acquires AdminConfig {
        AdminConfig[integrator_addr].addr
    }

    #[view]
    public fun get_pending_admin(integrator_addr: address): address acquires AdminConfig {
        AdminConfig[integrator_addr].pending_addr
    }

    #[view]
    public fun get_next_sequence(integrator_addr: address): u64 acquires IntegratorCapability {
        IntegratorCapability[integrator_addr].sequence
    }

    #[view]
    public fun get_transceivers(integrator_addr: address): vector<address> acquires TransceiversStore {
        TransceiversStore[integrator_addr].transceivers
    }

    #[view]
    public fun get_transceiver_index(integrator_addr: address, transceiver_addr: address): u8 acquires TransceiverToIndexStore {
        *table::borrow(&TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr)
    }

    #[view]
    public fun get_enabled_send_transceivers(integrator_addr: address, chain_id: u16): u128 acquires SendTransceiverStore {
        *table::borrow_with_default(&SendTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, &0)
    }

    #[view]
    public fun get_enabled_recv_transceivers(integrator_addr: address, chain_id: u16): u128 acquires RecvTransceiverStore {
        *table::borrow_with_default(&RecvTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, &0)
    }

    public entry fun update_admin(admin: &signer, integrator_addr: address, new_admin: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // If possible, MUST NOT allow the admin to discard admin via this command.
        assert!(new_admin != @0x0);
        // Immediately sets `newAdmin` as the admin of the integrator.
        admin_config.addr = new_admin;
    }

    public entry fun transfer_admin(admin: &signer, integrator_addr: address, new_admin: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // If possible, MUST NOT allow the admin to discard admin via this command.
        assert!(new_admin != @0x0);
        // Initiates the first step of a two-step process in which the current admin (to cancel) or new admin must claim.
        admin_config.pending_addr = new_admin;
    }
    
    public entry fun claim_admin(new_admin: &signer, integrator_addr: address) acquires AdminConfig {
        let admin_config = &mut AdminConfig[integrator_addr];
        // MUST check that there is an admin transfer pending.
        assert!(admin_config.pending_addr != @0x0);
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let new_admin_addr = signer::address_of(new_admin);
        assert!(new_admin_addr == admin_config.addr || new_admin_addr == admin_config.pending_addr);
        // Cancels / Completes the second step of the two-step transfer. Sets the admin to the caller and clears the pending admin.
        admin_config.addr = new_admin_addr;
        admin_config.pending_addr = @0x0;
    }

    public entry fun discard_admin(admin: &signer, integrator_addr: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // Clears the current admin. THIS IS NOT REVERSIBLE. This ensures that the Integrator configuration becomes immutable.
        admin_config.addr = @0x0;
    }

    public entry fun add_transceiver(admin: &signer, integrator_addr: address, transceiver_addr: address) acquires AdminConfig, TransceiversStore, TransceiverToIndexStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        let transceivers = &mut TransceiversStore[integrator_addr].transceivers;
        // MUST check that `transceiverAddr` is not already in the array.
        assert!(!vector::contains(transceivers, &transceiver_addr));
        // MUST check that the array would not surpass 128 entries.
        let len = vector::length(transceivers);
        assert!(len < MAX_TRANSCEIVERS);
        // Appends the `transceiverAddr` to the Integrator's array of Transceivers. THIS IS NOT REVERSIBLE. Once a transceiver is added for an Integrator, it cannot be removed.
        // Note: When a Transceiver is added, it is not enabled for sending or receiving on any chain.
        vector::push_back(transceivers, transceiver_addr);
        table::add(&mut TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr, (len as u8));
    }

    public entry fun enable_send_transceiver(admin: &signer, integrator_addr: address, chain_id: u16, transceiver_addr: address) acquires AdminConfig, TransceiverToIndexStore, SendTransceiverStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // MUST check that the `transceiverAddr` is in the Integrator's array of Transceivers.
        // The borrow will fail if the Transceiver was never added.
        let index = table::borrow(&TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr);
        // MUST check that the `transceiverAddr` is currently disabled for sending to the given chain.
        let bitmap = table::borrow_mut_with_default(&mut SendTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, 0);
        let bitmask = 1 << *index;
        assert!(*bitmap & bitmask == 0);
        // Enables the Transceiver for sending to the given chain.
        *bitmap = *bitmap | bitmask;
    }
    
    public entry fun disable_send_transceiver(admin: &signer, integrator_addr: address, chain_id: u16, transceiver_addr: address) acquires AdminConfig, TransceiverToIndexStore, SendTransceiverStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // MUST check that the `transceiverAddr` is in the Integrator's array of Transceivers.
        // The borrow will fail if the Transceiver was never added.
        let index = table::borrow(&TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr);
        // MUST check that the `transceiverAddr` is currently enabled for sending to the given chain.
        let bitmap = table::borrow_mut_with_default(&mut SendTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, 0);
        let bitmask = 1 << *index;
        assert!(*bitmap & bitmask > 0);
        // Disables the Transceiver for sending to the given chain.
        *bitmap = *bitmap & (bitmask ^ MAX_U128);
    }

    public entry fun enable_recv_transceiver(admin: &signer, integrator_addr: address, chain_id: u16, transceiver_addr: address) acquires AdminConfig, TransceiverToIndexStore, RecvTransceiverStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // MUST check that the `transceiverAddr` is in the Integrator's array of Transceivers.
        let index = table::borrow(&TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr);
        // MUST check that the `transceiverAddr` is currently disabled for receiving from the given chain.
        let bitmap = table::borrow_mut_with_default(&mut RecvTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, 0);
        let bitmask = 1 << *index;
        assert!(*bitmap & bitmask == 0);
        // Enables the Transceiver for receiving from the given chain.
        *bitmap = *bitmap | bitmask;
    }
    
    public entry fun disable_recv_transceiver(admin: &signer, integrator_addr: address, chain_id: u16, transceiver_addr: address) acquires AdminConfig, TransceiverToIndexStore, RecvTransceiverStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(signer::address_of(admin) == admin_config.addr);
        assert!(admin_config.pending_addr == @0x0);
        // MUST check that the `transceiverAddr` is in the Integrator's array of Transceivers.
        let index = table::borrow(&TransceiverToIndexStore[integrator_addr].transceiver_to_index, transceiver_addr);
        // MUST check that the `transceiverAddr` is currently enabled for receiving from the given chain.
        let bitmap = table::borrow_mut_with_default(&mut RecvTransceiverStore[integrator_addr].per_chain_transceiver_bitmap, chain_id, 0);
        let bitmask = 1 << *index;
        assert!(*bitmap & bitmask > 0);
        // Disables the Transceiver for receiving from the given chain.
        *bitmap = *bitmap & (bitmask ^ MAX_U128);
    }

    package fun use_sequence(integrator_acct: &signer): u64 acquires IntegratorCapability{
        let integrator_addr = signer::address_of(integrator_acct);
        let integrator_cap = &mut IntegratorCapability[integrator_addr];
        let sequence = integrator_cap.sequence;
        integrator_cap.sequence = sequence + 1;
        sequence
    }
}
