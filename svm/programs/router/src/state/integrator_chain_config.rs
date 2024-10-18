use anchor_lang::prelude::*;

use crate::utils::bitmap::Bitmap;

/// Manages the transceivers for a specific integrator on a particular chain.
///
/// This struct keeps track of both receive and send transceivers
/// using bitmaps for efficient storage and lookup.
#[account]
#[derive(InitSpace)]
pub struct IntegratorChainConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// The program ID of the integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Identifier for the blockchain
    /// This is used as a seed for PDA derivation
    pub chain_id: u16,

    /// Bitmap tracking the status of receive transceivers
    pub recv_transceiver_bitmap: Bitmap,

    /// Bitmap tracking the status of send transceivers
    pub send_transceiver_bitmap: Bitmap,
}

impl IntegratorChainConfig {
    /// Seed prefix for deriving IntegratorChainConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_config";

    pub fn new(bump: u8, integrator_program_id: Pubkey, chain_id: u16) -> Self {
        Self {
            bump,
            integrator_program_id,
            chain_id,
            recv_transceiver_bitmap: Bitmap::new(),
            send_transceiver_bitmap: Bitmap::new(),
        }
    }

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

    pub fn enable_recv_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
        self.recv_transceiver_bitmap
            .set(index, value)
            .map_err(|e| error!(e))
    }

    pub fn enable_send_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
        self.send_transceiver_bitmap
            .set(index, value)
            .map_err(|e| error!(e))
    }

    pub fn get_recv_transceiver(&self, index: u8) -> Result<bool> {
        self.recv_transceiver_bitmap
            .get(index)
            .map_err(|e| error!(e))
    }

    pub fn get_send_transceiver(&self, index: u8) -> Result<bool> {
        self.send_transceiver_bitmap
            .get(index)
            .map_err(|e| error!(e))
    }
}
