use crate::state::Config;
use anchor_lang::prelude::*;

/// Accounts struct for initializing the GMP Router
#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    /// The account paying for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Config account being initialized
    #[account(
        init,
        space = 8 + Config::INIT_SPACE,
        payer = payer,
        seeds = [Config::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, Config>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Arguments for the initialize instruction
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    /// The public key of the owner of the GMP Router
    pub owner: Pubkey,
}

/// Initializes the GMP Router by creating and setting up the Config account
///
/// This function creates the global configuration account for the GMP Router.
/// It sets the owner, initializes the program as unpaused, and sets the initial
/// integrator ID counter.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `args` - The initialization arguments, including the owner's public key
///
/// # Returns
///
/// Returns `Ok(())` if the initialization is successful
pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
    ctx.accounts.config.set_inner(Config {
        bump: ctx.bumps.config,
        owner: args.owner,
        paused: false,
        next_integrator_id: 0,
    });

    Ok(())
}
