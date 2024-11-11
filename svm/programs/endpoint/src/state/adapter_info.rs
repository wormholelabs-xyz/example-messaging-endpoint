use anchor_lang::prelude::*;

/// Represents a registered adapter in the Endpoint.
///
/// Each adapter is associated with a specific integrator and has a unique ID
/// within that integrator's context. It can be used across multiple chains.
#[account]
#[derive(InitSpace, Debug)]
pub struct AdapterInfo {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// The program ID of the integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Public key of the adapter's address
    /// This is used as a seed for PDA derivation
    pub adapter_program_id: Pubkey,

    /// Index of the adapter with respect to the adapter_info vector in
    /// IntegratorConfig
    pub index: u8,
}

impl AdapterInfo {
    /// Seed prefix for deriving AdapterInfo PDAs
    pub const SEED_PREFIX: &'static [u8] = b"adapter_info";

    pub fn pda(integrator_program_id: &Pubkey, adapter_program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                integrator_program_id.as_ref(),
                adapter_program_id.as_ref(),
            ],
            &crate::ID,
        )
    }
}
