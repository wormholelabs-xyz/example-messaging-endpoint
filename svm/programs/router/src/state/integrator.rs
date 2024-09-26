use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Integrator {
    pub bump: u8,
    pub id: u64,
    pub authority: Pubkey,
}

impl Integrator {
    pub const SEED_PREFIX: &'static [u8] = b"integrator";
}
