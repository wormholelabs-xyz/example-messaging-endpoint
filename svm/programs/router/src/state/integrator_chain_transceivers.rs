use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct IntegratorChainTransceivers {
    pub bump: u8,
    pub integrator_id: u64,
    pub chain_id: u16,
    pub next_transceiver_id: u64,
    pub transceiver_bitmap: u64,
}

impl IntegratorChainTransceivers {
    pub const SEED_PREFIX: &'static [u8] = b"integrator_chain_transceivers";
    pub const MAX_TRANSCEIVERS: usize = 64;
}
