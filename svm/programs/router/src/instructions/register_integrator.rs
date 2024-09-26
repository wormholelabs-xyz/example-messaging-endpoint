use crate::error::RouterError;
use crate::state::{Config, Integrator};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterIntegrator<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
        constraint = !config.paused @ RouterError::ProgramPaused,
    )]
    pub config: Account<'info, Config>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Integrator::INIT_SPACE,
        seeds = [
            Integrator::SEED_PREFIX,
            config.next_integrator_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub integrator: Account<'info, Integrator>,

    pub system_program: Program<'info, System>,
}

pub fn register_integrator(ctx: Context<RegisterIntegrator>, authority: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let integrator_id = config.next_integrator_id;
    config.next_integrator_id += 1;

    ctx.accounts.integrator.set_inner(Integrator {
        bump: ctx.bumps.integrator,
        id: integrator_id,
        authority,
    });

    Ok(())
}
