use anchor_lang::prelude::*;

use crate::utils::bitmap::Bitmap;

/// Manages the transceivers for a specific integrator on a particular chain.
///
/// This struct keeps track of both incoming and outgoing transceivers
/// using bitmaps and counters for efficient storage and lookup.
///
/// Note: While separating incoming and outgoing transceiver data into
/// different accounts could improve parallelism, we've chosen to keep
/// them together to save on account creation and storage costs. This
/// is based on the expectation that transceiver registration is a
/// low-frequency operation, making the potential parallelism benefits
/// less significant than the account efficiency gains.
#[account]
#[derive(InitSpace)]
pub struct IntegratorChainTransceivers {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Identifier for the blockchain
    pub chain_id: u16,

    /// Counter for assigning IDs to incoming transceivers
    pub next_in_transceiver_id: u8,

    /// Counter for assigning IDs to outgoing transceivers
    pub next_out_transceiver_id: u8,

    /// Bitmap tracking the status of incoming transceivers
    pub in_transceiver_bitmap: Bitmap,

    /// Bitmap tracking the status of outgoing transceivers
    pub out_transceiver_bitmap: Bitmap,
}

impl IntegratorChainTransceivers {
    /// Seed prefix for deriving IntegratorChainTransceivers PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_transceivers";

    /// Maximum number of transceivers allowed per direction (in/out)
    pub const MAX_TRANSCEIVERS: u8 = 128;

    pub fn set_in_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
        self.in_transceiver_bitmap
            .set(index, value)
            .map_err(|e| error!(e))
    }

    pub fn set_out_transceiver(&mut self, index: u8, value: bool) -> Result<()> {
        self.out_transceiver_bitmap
            .set(index, value)
            .map_err(|e| error!(e))
    }

    pub fn get_in_transceiver(&self, index: u8) -> Result<bool> {
        self.in_transceiver_bitmap.get(index).map_err(|e| error!(e))
    }

    pub fn get_out_transceiver(&self, index: u8) -> Result<bool> {
        self.out_transceiver_bitmap
            .get(index)
            .map_err(|e| error!(e))
    }
}
