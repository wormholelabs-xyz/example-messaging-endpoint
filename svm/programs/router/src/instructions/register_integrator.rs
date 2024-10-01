use crate::error::RouterError;
use crate::state::{Config, Integrator};
use anchor_lang::prelude::*;

/// Accounts struct for registering a new integrator
#[derive(Accounts)]
pub struct RegisterIntegrator<'info> {
    /// The global configuration account
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
        constraint = !config.paused @ RouterError::ProgramPaused,
    )]
    pub config: Account<'info, Config>,

    /// The owner of the GMP Router
    pub owner: Signer<'info>,

    /// The account paying for the registration
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Integrator account being initialized
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

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Registers a new integrator in the GMP Router
///
/// This function creates a new Integrator account and assigns it a unique ID.
/// It also updates the global configuration to increment the integrator ID counter.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `authority` - The public key of the authority controlling this integrator
///
/// # Returns
///
/// Returns `Ok(())` if the registration is successful
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
