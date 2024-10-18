use crate::error::RouterError;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateAdminArgs {
    /// The new_admin to be assigned
    pub new_admin: Pubkey,

    /// The integrator_program for the integrator_config
    pub integrator_program_id: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: UpdateAdminArgs)]
pub struct UpdateAdmin<'info> {
    /// The current admin of the IntegratorConfig account
    pub admin: Signer<'info>,

    /// The IntegratorConfig account being transferred
    /// `has_one` constraint checks that the signer is the current admin
    #[account(
        mut,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            args.integrator_program_id.key().as_ref(),
        ],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::CallerNotAuthorized,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,
}

impl<'info> UpdateAdmin<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}

/// Updates the admin of an IntegratorConfig account.
///
/// This function transfers the administration of an IntegratorConfig account from the current admin
/// to a new admin. It checks that the current admin is the signer of the transaction and updates
/// the admin field in the IntegratorConfig account.
///
/// # Arguments
///
/// * `ctx` - The context of the request, containing the accounts involved in the admin update.
/// * `args` - The UpdateAdminArg struct containing the new admin's public key.
///
/// # Returns
///
/// Returns `Ok(())` if the admin update is successful, otherwise returns an error.
#[access_control(UpdateAdmin::validate(&ctx.accounts))]
pub fn update_admin(ctx: Context<UpdateAdmin>, args: UpdateAdminArgs) -> Result<()> {
    // Check if there's a pending admin transfer
    if ctx.accounts.integrator_config.pending_admin.is_some() {
        return Err(RouterError::AdminTransferInProgress.into());
    }

    ctx.accounts
        .integrator_config
        .update_admin(args.new_admin)?;

    Ok(())
}
