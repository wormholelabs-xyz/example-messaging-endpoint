use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

/// Accounts struct for initializing an IntegratorConfig account
#[derive(Accounts)]
pub struct InitIntegratorConfig<'info> {
    /// The account paying for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The authority (owner) of the IntegratorConfig account
    /// CHECK: The integrator program is responsible for passing the correct authority
    pub authority: UncheckedAccount<'info>,

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

    /// The integrator program
    pub integrator_program: Signer<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Initializes an IntegratorConfig account for a specific integrator_program
///
/// This function sets up the initial state for managing the configuration
/// of a given integrator_program. It initializes the authority, program ID,
/// and the transceiver ID counter.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
///
/// # Returns
///
/// Returns `Ok(())` if the initialization is successful
pub fn init_integrator_config(ctx: Context<InitIntegratorConfig>) -> Result<()> {
    msg!(
        "Initializing IntegratorConfig for program: {}",
        ctx.accounts.integrator_program.key()
    );

    ctx.accounts.integrator_config.set_inner(IntegratorConfig {
        bump: ctx.bumps.integrator_config,
        authority: ctx.accounts.authority.key(),
        program_id: ctx.accounts.integrator_program.key(),
        next_transceiver_id: 0,
    });

    msg!("IntegratorConfig initialized successfully");
    Ok(())
}
