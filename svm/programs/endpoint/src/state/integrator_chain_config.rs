use anchor_lang::prelude::*;

use crate::utils::bitmap::Bitmap;

/// Manages the adapters for a specific integrator on a particular chain.
///
/// This struct keeps track of both receive and send adapters
/// using bitmaps for efficient storage and lookup.
#[account]
#[derive(InitSpace, Debug)]
pub struct IntegratorChainConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// The program ID of the integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Identifier for the blockchain
    /// This is used as a seed for PDA derivation
    pub chain_id: u16,

    /// Bitmap tracking the status of send adapters
    pub send_adapter_bitmap: Bitmap,

    /// Bitmap tracking the status of receive adapters
    pub recv_adapter_bitmap: Bitmap,
}

impl IntegratorChainConfig {
    /// Seed prefix for deriving IntegratorChainConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_config";

    pub fn pda(integrator_program: &Pubkey, chain_id: u16) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                integrator_program.as_ref(),
                chain_id.to_be_bytes().as_ref(),
            ],
            &crate::ID,
        )
    }
}
