use anchor_lang::event;
use anchor_lang::prelude::*;

/// Event emitted when a new integrator is registered
#[event]
pub struct IntegratorRegistered {
    pub integrator: Pubkey,
    pub admin: Pubkey,
}

/// Event emitted when an integrator's admin is updated
#[event]
pub struct AdminUpdated {
    pub integrator: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}

/// Event emitted when an admin update is requested
#[event]
pub struct AdminUpdateRequested {
    pub integrator: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}

/// Event emitted when a message is sent
#[event]
pub struct MessageSent {
    pub sender: [u8; 32],
    pub sequence: u64,
    pub recipient: [u8; 32],
    pub recipient_chain: u16,
    pub payload_digest: [u8; 32],
}

/// Event emitted when a message is picked up by an adapter
#[event]
pub struct MessagePickedUp {
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: [u8; 32],
    pub payload_hash: [u8; 32],
    pub adapter: Pubkey,
    pub remaining_adapters: u128,
}

/// Event emitted when a message is attested to by an adapter
#[event]
pub struct MessageAttestedTo {
    pub message_hash: [u8; 32],
    pub src_chain: u16,
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: [u8; 32],
    pub payload_hash: [u8; 32],
    pub attested_bitmap: u128,
    pub attesting_adapter: [u8; 32],
}

/// Event emitted when a message is received
#[event]
pub struct MessageReceived {
    pub message_hash: [u8; 32],
    pub src_chain: u16,
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: [u8; 32],
    pub payload_hash: [u8; 32],
    pub enabled_bitmap: u128,
    pub attested_bitmap: u128,
}

/// Event emitted when a message is executed
#[event]
pub struct MessageExecuted {
    pub message_hash: [u8; 32],
    pub src_chain: u16,
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: [u8; 32],
    pub payload_hash: [u8; 32],
}

/// Event emitted when a new adapter is added to an integrator
#[event]
pub struct AdapterAdded {
    pub integrator: Pubkey,
    pub adapter: Pubkey,
    pub adapters_num: u8,
}

/// Event emitted when a send adapter is enabled for a specific chain
#[event]
pub struct SendAdapterEnabledForChain {
    pub integrator: Pubkey,
    pub chain: u16,
    pub adapter: Pubkey,
}

/// Event emitted when a receive adapter is enabled for a specific chain
#[event]
pub struct RecvAdapterEnabledForChain {
    pub integrator: Pubkey,
    pub chain: u16,
    pub adapter: Pubkey,
}

/// Event emitted when a send adapter is disabled for a specific chain
#[event]
pub struct SendAdapterDisabledForChain {
    pub integrator: Pubkey,
    pub chain: u16,
    pub adapter: Pubkey,
}

/// Event emitted when a receive adapter is disabled for a specific chain
#[event]
pub struct RecvAdapterDisabledForChain {
    pub integrator: Pubkey,
    pub chain: u16,
    pub adapter: Pubkey,
}

/// Event emitted when an admin is discarded for an integrator
#[event]
pub struct AdminDiscarded {
    pub integrator: Pubkey,
}
