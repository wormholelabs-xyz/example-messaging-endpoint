use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Integrator {
    pub bump: u8,
    pub id: u16,
    pub authority: Pubkey,
    pub next_transceiver_id: u16,
}

impl Integrator {
    pub const SEED_PREFIX: &'static [u8] = b"integrator";
}
