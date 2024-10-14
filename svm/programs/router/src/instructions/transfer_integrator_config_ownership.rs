use crate::error::RouterError;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferIntegratorConfigOwnership<'info> {
    /// The current owner of the IntegratorConfig account
    pub admin: Signer<'info>,

    /// The new owner of the IntegratorConfig account
    pub new_admin: Signer<'info>,

    /// The IntegratorConfig account being transferred
    #[account(
        mut,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            integrator_program.key().as_ref(),
        ],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
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
        ctx.accounts.admin.key(),
        ctx.accounts.new_admin.key()
    );

    ctx.accounts
        .integrator_config
        .update_admin(&ctx.accounts.admin, ctx.accounts.new_admin.key())?;

    msg!("IntegratorConfig ownership transferred successfully");
    Ok(())
}
