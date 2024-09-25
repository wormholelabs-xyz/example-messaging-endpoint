use anchor_lang::prelude::*;
use crate::state::Config;

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        space = 8 + Config::INIT_SPACE,
        payer = payer,
        seeds = [Config::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub owner: Pubkey,
}

pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
    ctx.accounts.config.set_inner(Config {
        bump: ctx.bumps.config,
        owner: args.owner,
        paused: false,
        next_integrator_id: 0,
    });

    Ok(())
}
