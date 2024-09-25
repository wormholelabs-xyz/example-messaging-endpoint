use crate::error::RouterError;
use crate::state::{Config, Integrator, RegisteredTransceiver};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterTransceiver<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
        constraint = !config.paused @ RouterError::ProgramPaused,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [Integrator::SEED_PREFIX, integrator.id.to_le_bytes().as_ref()],
        bump = integrator.bump,
        has_one = authority @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator: Account<'info, Integrator>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + RegisteredTransceiver::INIT_SPACE,
        seeds = [
            RegisteredTransceiver::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            integrator.next_transceiver_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    transceiver_address: Pubkey,
) -> Result<()> {
    let integrator = &mut ctx.accounts.integrator;
    let transceiver_id = integrator.next_transceiver_id;
    integrator.next_transceiver_id += 1;

    ctx.accounts
        .registered_transceiver
        .set_inner(RegisteredTransceiver {
            bump: ctx.bumps.registered_transceiver,
            integrator_id: integrator.id,
            id: transceiver_id,
            address: transceiver_address,
        });

    Ok(())
}
