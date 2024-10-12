use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

/// Accounts struct for initializing an IntegratorConfig account
#[derive(Accounts)]
pub struct InitIntegratorConfig<'info> {
    /// The account paying for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The owner of the IntegratorConfig account
    /// TODO: check if this should be a signer
    /// CHECK: The integrator program is responsible for passing the correct owner
    pub owner: UncheckedAccount<'info>,

    /// The IntegratorConfig account being initialized
    #[account(
        init,
        payer = payer,
        space = 8 + IntegratorConfig::INIT_SPACE,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            integrator_program.key().as_ref(),
        ],
        bump
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The integrator program (must be a signer)
    pub integrator_program: Signer<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

pub fn init_integrator_config(ctx: Context<InitIntegratorConfig>) -> Result<()> {
    msg!(
        "Initializing IntegratorConfig for program: {}",
        ctx.accounts.integrator_program.key()
    );

    ctx.accounts.integrator_config.set_inner(IntegratorConfig {
        bump: ctx.bumps.integrator_config,
        owner: ctx.accounts.owner.key(),
        integrator_program_id: ctx.accounts.integrator_program.key(),
        transceivers: Vec::new(),
    });

    msg!("IntegratorConfig initialized successfully");
    Ok(())
}
