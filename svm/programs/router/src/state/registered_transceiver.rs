use anchor_lang::prelude::*;

/// Represents a registered transceiver in the GMP Router.
///
/// Each transceiver is associated with a specific integrator and has a unique ID
/// within that integrator's context. It can be used across multiple chains.
#[account]
#[derive(InitSpace)]
pub struct RegisteredTransceiver {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Unique identifier for the transceiver within its integrator context
    pub id: u8,

    /// The program ID of the integrator
    pub integrator_program_id: Pubkey,

    /// Public key of the transceiver's address
    pub address: Pubkey,
}

impl RegisteredTransceiver {
    /// Seed prefix for deriving RegisteredTransceiver PDAs
    pub const SEED_PREFIX: &'static [u8] = b"registered_transceiver";

    pub fn pda(integrator_program_id: &Pubkey, id: u8) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, integrator_program_id.as_ref(), &[id]],
            &crate::ID,
        )
    }
}
