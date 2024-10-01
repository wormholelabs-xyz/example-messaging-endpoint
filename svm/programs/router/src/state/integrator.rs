use anchor_lang::prelude::*;

/// Represents an entity that can interact with the GMP Router.
///
/// Each Integrator has a unique ID and an associated authority
/// that can perform actions on its behalf.
#[account]
#[derive(InitSpace)]
pub struct Integrator {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Unique identifier for the integrator
    pub id: u64,

    /// Public key of the authority controlling this integrator
    pub authority: Pubkey,
}

impl Integrator {
    /// Seed prefix for deriving Integrator PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator";
}
