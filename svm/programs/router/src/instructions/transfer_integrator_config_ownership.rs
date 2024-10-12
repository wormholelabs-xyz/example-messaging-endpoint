use crate::error::RouterError;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferIntegratorConfigOwnership<'info> {
    /// The current owner of the IntegratorConfig account
    pub owner: Signer<'info>,

    /// The new owner of the IntegratorConfig account
    pub new_owner: Signer<'info>,

    /// The IntegratorConfig account being transferred
    #[account(
        mut,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            integrator_program.key().as_ref(),
        ],
        bump = integrator_config.bump,
        has_one = owner @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The integrator program
    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,
}

pub fn transfer_integrator_config_ownership(
    ctx: Context<TransferIntegratorConfigOwnership>,
) -> Result<()> {
    msg!(
        "Transferring IntegratorConfig ownership from {} to {}",
        ctx.accounts.owner.key(),
        ctx.accounts.new_owner.key()
    );

    ctx.accounts
        .integrator_config
        .transfer_owner(&ctx.accounts.owner, ctx.accounts.new_owner.key())?;

    msg!("IntegratorConfig ownership transferred successfully");
    Ok(())
}
