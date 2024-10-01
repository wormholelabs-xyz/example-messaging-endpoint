use crate::{
    state::{Config, Integrator, IntegratorChainTransceivers},
    utils::bitmap::Bitmap,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(chain_id: u16)]
pub struct InitIntegratorChainTransceivers<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [Integrator::SEED_PREFIX, integrator.id.to_le_bytes().as_ref()],
        bump = integrator.bump,
    )]
    pub integrator: Account<'info, Integrator>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + IntegratorChainTransceivers::INIT_SPACE,
        seeds = [
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,

    pub system_program: Program<'info, System>,
}

pub fn init_integrator_chain_transceivers(
    ctx: Context<InitIntegratorChainTransceivers>,
    chain_id: u16,
) -> Result<()> {
    let chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;
    chain_transceivers.integrator_id = ctx.accounts.integrator.id;
    chain_transceivers.chain_id = chain_id;
    chain_transceivers.next_in_transceiver_id = 0;
    chain_transceivers.next_out_transceiver_id = 0;
    chain_transceivers.in_transceiver_bitmap = Bitmap::new();
    chain_transceivers.out_transceiver_bitmap = Bitmap::new();

    Ok(())
}
