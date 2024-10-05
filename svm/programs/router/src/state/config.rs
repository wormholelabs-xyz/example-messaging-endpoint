use anchor_lang::prelude::*;

/// Represents the global configuration of the GMP Router.
///
/// This struct stores important global state including:
/// - A counter for assigning unique IDs to integrators
///
/// The Config account is a singleton, created during program initialization.
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Counter for assigning unique IDs to integrators
    pub next_integrator_id: u64,
}

impl Config {
    /// Seed prefix for deriving the Config PDA
    pub const SEED_PREFIX: &'static [u8] = b"config";
}
