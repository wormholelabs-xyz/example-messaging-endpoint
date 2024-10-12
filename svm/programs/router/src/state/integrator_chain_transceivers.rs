use anchor_lang::prelude::*;

use crate::utils::bitmap::Bitmap;

/// Manages the transceivers for a specific integrator on a particular chain.
///
/// This struct keeps track of both receive and send transceivers
/// using bitmaps for efficient storage and lookup.
#[account]
#[derive(InitSpace)]
pub struct IntegratorChainTransceivers {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Identifier for the blockchain
    pub chain_id: u16,

    /// The program ID of the integrator
    pub integrator_program_id: Pubkey,

    /// Bitmap tracking the status of receive transceivers
    pub recv_transceiver_bitmap: Bitmap,

    /// Bitmap tracking the status of send transceivers
    pub send_transceiver_bitmap: Bitmap,
}

impl IntegratorChainTransceivers {
    /// Seed prefix for deriving IntegratorChainTransceivers PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_transceivers";

    /// Maximum number of transceivers allowed per direction (recv/send)
    pub const MAX_TRANSCEIVERS: u8 = 128;

    pub fn new(bump: u8, chain_id: u16, integrator_program_id: Pubkey) -> Self {
        Self {
            bump,
            chain_id,
            integrator_program_id,
            recv_transceiver_bitmap: Bitmap::new(),
            send_transceiver_bitmap: Bitmap::new(),
        }
    }

    pub fn set_recv_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
        self.recv_transceiver_bitmap
            .set(index, value)
            .map_err(|e| error!(e))
    }

    pub fn set_send_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
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
