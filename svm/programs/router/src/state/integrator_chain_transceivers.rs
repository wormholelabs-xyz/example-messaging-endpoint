use anchor_lang::prelude::*;

use crate::utils::bitmap::Bitmap;

#[account]
#[derive(InitSpace)]
pub struct IntegratorChainTransceivers {
    pub bump: u8,
    pub integrator_id: u64,
    pub chain_id: u16,
    pub next_in_transceiver_id: u64,
    pub next_out_transceiver_id: u64,
    pub in_transceiver_bitmap: Bitmap,
    pub out_transceiver_bitmap: Bitmap,
}

impl IntegratorChainTransceivers {
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_transceivers";
    pub const MAX_TRANSCEIVERS: usize = 64;

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
