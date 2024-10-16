use crate::error::RouterError;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    /// The current admin of the IntegratorConfig account
    pub admin: Signer<'info>,

    /// The new admin of the IntegratorConfig account
    /// CHECK: The integrator program is responsible for passing the correct admin
    pub new_admin: UncheckedAccount<'info>,

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

/// Updates the admin of an IntegratorConfig account.
///
/// This function transfers the adminship of an IntegratorConfig account from the current admin
/// to a new admin. It checks that the current admin is the signer of the transaction and updates
/// the admin field in the IntegratorConfig account.
///
/// # Arguments
///
/// * `ctx` - The context of the request, containing the accounts involved in the admin update.
///
/// # Returns
///
/// Returns `Ok(())` if the admin update is successful, otherwise returns an error.
pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> {
    msg!(
        "Transferring IntegratorConfig admin from {} to {}",
        ctx.accounts.admin.key(),
        ctx.accounts.new_admin.key()
    );

    ctx.accounts
        .integrator_config
        .update_admin(&ctx.accounts.admin, ctx.accounts.new_admin.key())?;

    msg!("IntegratorConfig adminship transferred successfully");
    Ok(())
}
