use anchor_lang::prelude::*;

/// Represents a registered transceiver in the GMP Router.
///
/// Each transceiver is associated with a specific integrator and has a unique ID
/// within that integrator's context. It can be used across multiple chains.
#[account]
#[derive(InitSpace, Debug)]
pub struct TransceiverInfo {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// The program ID of the integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Public key of the transceiver's address
    /// This is used as a seed for PDA derivation
    pub transceiver_program_id: Pubkey,

    /// Index of the transceiver with respect to the registered_transceiver vector in
    /// IntegratorConfig
    pub index: u8,
}

impl TransceiverInfo {
    /// Seed prefix for deriving TransceiverInfo PDAs
    pub const SEED_PREFIX: &'static [u8] = b"transceiver_info";

    pub fn pda(integrator_program_id: &Pubkey, transceiver_program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                integrator_program_id.as_ref(),
                transceiver_program_id.as_ref(),
            ],
            &crate::ID,
        )
    }
}
