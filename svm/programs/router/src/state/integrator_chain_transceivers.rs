use anchor_lang::prelude::*;

#[account]
pub struct IntegratorChainTransceivers {
    pub bump: u8,
    pub integrator_id: u64,
    pub chain_id: u64,
    pub next_transceiver_id: u64,
    pub transceiver_bitmap: [u64; 4], // support up to 256 transceivers
}

impl IntegratorChainTransceivers {
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_transceivers";
    pub const INIT_SPACE: usize = 8 + 1 + 8 + 8 + 8 + (8 * 4); // 8 (discriminator) + 1 + 8 + 8 + 8 + 32 bytes
}
