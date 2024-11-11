module endpoint::integrator {
    use endpoint::bitmap;
    use std::option::{Self, Option};
    use std::signer;
    use std::table::{Self, Table};
    use std::vector;

    const MAX_ADAPTERS: u64 = 128;

    const E_ALREADY_REGISTERED: u64 = 0;
    const E_INVALID_ADMIN: u64 = 1;
    const E_NOT_AUTHORIZED: u64 = 2;
    const E_ADMIN_TRANSFER_IN_PROGRESS: u64 = 3;
    const E_NO_ADMIN_TRANSFER_IN_PROGRESS: u64 = 4;
    const E_MAX_ADAPTERS_REACHED: u64 = 5;

    struct IntegratorCapability has key, store {
        /// Sequence number of the next message
        sequence: u64
    }

    struct AdminConfig has key, store {
        // The address of the admin
        admin_addr: Option<address>,
        // The address of the pending admin
        pending_admin_addr: Option<address>
    }

    struct AdaptersStore has key, store {
        adapters: vector<address>
    }

    struct AdapterToIndexStore has key, store {
        adapter_to_index: Table<address, u8>
    }

    struct SendAdapterStore has key, store {
        per_chain_adapter_bitmap: Table<u16, u128>
    }

    struct RecvAdapterStore has key, store {
        per_chain_adapter_bitmap: Table<u16, u128>
    }

    package fun new_integrator(integrator_acct: &signer, admin_addr: address) {
        let integrator_addr = signer::address_of(integrator_acct);
        // MUST check that the caller (Integrator) is not already registered.
        assert!(!exists<IntegratorCapability>(integrator_addr), E_ALREADY_REGISTERED);
        // If possible, MUST check that the admin is potentially valid / non-null.
        assert!(admin_addr != @0x0, E_INVALID_ADMIN);
        // Initializes their registration and sets the initial admin.
        move_to(integrator_acct, IntegratorCapability { sequence: 0 });
        move_to(integrator_acct, AdminConfig{admin_addr: option::some(admin_addr), pending_admin_addr: option::none()});
        move_to(integrator_acct, AdaptersStore{adapters: vector::empty()});
        move_to(integrator_acct, AdapterToIndexStore{adapter_to_index: table::new<address, u8>()});
        move_to(integrator_acct, SendAdapterStore{per_chain_adapter_bitmap: table::new<u16, u128>()});
        move_to(integrator_acct, RecvAdapterStore{per_chain_adapter_bitmap: table::new<u16, u128>()});
    }

    #[view]
    public fun get_admin(integrator_addr: address): Option<address> acquires AdminConfig {
        AdminConfig[integrator_addr].admin_addr
    }

    #[view]
    public fun get_pending_admin(integrator_addr: address): Option<address> acquires AdminConfig {
        AdminConfig[integrator_addr].pending_admin_addr
    }

    #[view]
    public fun get_next_sequence(integrator_addr: address): u64 acquires IntegratorCapability {
        IntegratorCapability[integrator_addr].sequence
    }

    #[view]
    public fun get_adapters(integrator_addr: address): vector<address> acquires AdaptersStore {
        AdaptersStore[integrator_addr].adapters
    }

    #[view]
    public fun get_adapter_index(integrator_addr: address, adapter_addr: address): u8 acquires AdapterToIndexStore {
        *table::borrow(&AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr)
    }

    #[view]
    public fun get_enabled_send_adapters(integrator_addr: address, chain_id: u16): u128 acquires SendAdapterStore {
        *table::borrow_with_default(&SendAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, &0)
    }

    #[view]
    public fun get_enabled_recv_adapters(integrator_addr: address, chain_id: u16): u128 acquires RecvAdapterStore {
        *table::borrow_with_default(&RecvAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, &0)
    }

    public entry fun update_admin(admin: &signer, integrator_addr: address, new_admin: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // If possible, MUST NOT allow the admin to discard admin via this command.
        assert!(new_admin != @0x0, E_INVALID_ADMIN);
        // Immediately sets `newAdmin` as the admin of the integrator.
        admin_config.admin_addr.swap(new_admin);
    }

    public entry fun transfer_admin(admin: &signer, integrator_addr: address, new_admin: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // If possible, MUST NOT allow the admin to discard admin via this command.
        assert!(new_admin != @0x0, E_INVALID_ADMIN);
        // Initiates the first step of a two-step process in which the current admin (to cancel) or new admin must claim.
        // .fill will ensure there was not a pending transfer, as it must be empty in order to fill
        admin_config.pending_admin_addr.fill(new_admin);
    }
    
    public entry fun claim_admin(new_admin: &signer, integrator_addr: address) acquires AdminConfig {
        let admin_config = &mut AdminConfig[integrator_addr];
        // MUST check that there is an admin transfer pending.
        assert!(admin_config.pending_admin_addr.is_some(), E_NO_ADMIN_TRANSFER_IN_PROGRESS);
        // MUST check that the caller is the current admin OR the pending admin.
        let new_admin_addr = signer::address_of(new_admin);
        assert!(admin_config.admin_addr.contains(&new_admin_addr) || admin_config.pending_admin_addr.contains(&new_admin_addr), E_NOT_AUTHORIZED);
        // Cancels / Completes the second step of the two-step transfer. Sets the admin to the caller and clears the pending admin.
        admin_config.admin_addr.swap(new_admin_addr);
        // .extract requires that this contains a value
        admin_config.pending_admin_addr.extract();
    }

    public entry fun discard_admin(admin: &signer, integrator_addr: address) acquires AdminConfig {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &mut AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // Clears the current admin. THIS IS NOT REVERSIBLE. This ensures that the Integrator configuration becomes immutable.
        admin_config.admin_addr.extract();
    }

    public entry fun add_adapter(admin: &signer, integrator_addr: address, adapter_addr: address) acquires AdminConfig, AdaptersStore, AdapterToIndexStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        let adapters = &mut AdaptersStore[integrator_addr].adapters;
        // MUST check that `adapterAddr` is not already in the array.
        assert!(!vector::contains(adapters, &adapter_addr), E_ALREADY_REGISTERED);
        // MUST check that the array would not surpass 128 entries.
        let len = vector::length(adapters);
        assert!(len < MAX_ADAPTERS, E_MAX_ADAPTERS_REACHED);
        // Appends the `adapterAddr` to the Integrator's array of Adapters. THIS IS NOT REVERSIBLE. Once an adapter is added for an Integrator, it cannot be removed.
        // Note: When an Adapter is added, it is not enabled for sending or receiving on any chain.
        vector::push_back(adapters, adapter_addr);
        table::add(&mut AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr, (len as u8));
    }

    public entry fun enable_send_adapter(admin: &signer, integrator_addr: address, chain_id: u16, adapter_addr: address) acquires AdminConfig, AdapterToIndexStore, SendAdapterStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // MUST check that the `adapterAddr` is in the Integrator's array of Adapters.
        // The borrow will fail if the Adapter was never added.
        let index = table::borrow(&AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr);
        let bitmap = table::borrow_mut_with_default(&mut SendAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, 0);
        // MUST check that the `adapterAddr` is currently disabled for sending to the given chain.
        // Enables the Adapter for sending to the given chain.
        *bitmap = bitmap::enable(*bitmap, *index);
    }
    
    public entry fun disable_send_adapter(admin: &signer, integrator_addr: address, chain_id: u16, adapter_addr: address) acquires AdminConfig, AdapterToIndexStore, SendAdapterStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // MUST check that the `adapterAddr` is in the Integrator's array of Adapters.
        // The borrow will fail if the Adapter was never added.
        let index = table::borrow(&AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr);
        let bitmap = table::borrow_mut_with_default(&mut SendAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, 0);
        // MUST check that the `adapterAddr` is currently enabled for sending to the given chain.
        // Disables the Adapter for sending to the given chain.
        *bitmap = bitmap::disable(*bitmap, *index);
    }

    public entry fun enable_recv_adapter(admin: &signer, integrator_addr: address, chain_id: u16, adapter_addr: address) acquires AdminConfig, AdapterToIndexStore, RecvAdapterStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // MUST check that the `adapterAddr` is in the Integrator's array of Adapters.
        let index = table::borrow(&AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr);
        let bitmap = table::borrow_mut_with_default(&mut RecvAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, 0);
        // MUST check that the `adapterAddr` is currently disabled for receiving from the given chain.
        // Enables the Adapter for receiving from the given chain.
        *bitmap = bitmap::enable(*bitmap, *index);
    }
    
    public entry fun disable_recv_adapter(admin: &signer, integrator_addr: address, chain_id: u16, adapter_addr: address) acquires AdminConfig, AdapterToIndexStore, RecvAdapterStore {
        // MUST check that the caller is the current admin and there is not a pending transfer.
        let admin_config = &AdminConfig[integrator_addr];
        assert!(admin_config.admin_addr.contains(&signer::address_of(admin)), E_NOT_AUTHORIZED);
        assert!(admin_config.pending_admin_addr.is_none(), E_ADMIN_TRANSFER_IN_PROGRESS);
        // MUST check that the `adapterAddr` is in the Integrator's array of Adapters.
        let index = table::borrow(&AdapterToIndexStore[integrator_addr].adapter_to_index, adapter_addr);
        let bitmap = table::borrow_mut_with_default(&mut RecvAdapterStore[integrator_addr].per_chain_adapter_bitmap, chain_id, 0);
        // MUST check that the `adapterAddr` is currently enabled for receiving from the given chain.
        // Disables the Adapter for receiving from the given chain.
        *bitmap = bitmap::disable(*bitmap, *index);
    }

    package fun use_sequence(integrator_acct: &signer): u64 acquires IntegratorCapability{
        let integrator_addr = signer::address_of(integrator_acct);
        let integrator_cap = &mut IntegratorCapability[integrator_addr];
        let sequence = integrator_cap.sequence;
        integrator_cap.sequence = sequence + 1;
        sequence
    }
}
