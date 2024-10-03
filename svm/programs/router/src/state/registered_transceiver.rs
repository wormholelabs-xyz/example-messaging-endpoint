use anchor_lang::prelude::*;

/// Represents a registered transceiver in the GMP Router.
///
/// Each transceiver is associated with a specific integrator and chain,
/// and has a unique ID within that context.
#[account]
#[derive(InitSpace)]
pub struct RegisteredTransceiver {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Unique identifier for the transceiver within its integrator and chain context
    pub id: u8,

    /// Identifier for the blockchain this transceiver operates on
    pub chain_id: u16,

    /// Public key of the transceiver's address
    pub address: Pubkey,
}

impl RegisteredTransceiver {
    /// Seed prefix for deriving RegisteredTransceiver PDAs
    pub const SEED_PREFIX: &'static [u8] = b"registered_transceiver";
}
