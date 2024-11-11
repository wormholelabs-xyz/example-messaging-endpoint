use crate::error::EndpointError;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferAdminArgs {
    /// The new_admin to be assigned
    pub new_admin: Pubkey,

    /// The integrator_program for the integrator_config
    pub integrator_program_id: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: TransferAdminArgs)]
pub struct TransferAdmin<'info> {
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
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,
}

impl<'info> TransferAdmin<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}

#[derive(Accounts)]
pub struct ClaimAdmin<'info> {
    /// The signer, which must be the pending_admin
    pub new_admin: Signer<'info>,

    /// The IntegratorConfig account being claimed
    /// The constraint here checks that there is a pending admin transfer and the signer is the pending_admin
    #[account(
        mut,
        constraint = integrator_config.admin.is_some() @ EndpointError::CallerNotAuthorized,
        constraint = integrator_config.pending_admin.is_some() @ EndpointError::NoAdminTransferInProgress,
        constraint = integrator_config.pending_admin == Some(new_admin.key())
        || integrator_config.admin == Some(new_admin.key()) @ EndpointError::CallerNotAuthorized,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,
}

/// Initiates the transfer of admin rights for an IntegratorConfig account.
///
/// This function sets a pending admin for the IntegratorConfig account. The pending admin
/// must later claim the admin rights using the `claim_admin` function.
///
/// If there is already a transfer in progress (in other words `pending_admin` is not null)
/// `AdminTransferInProgress` will be returned
///
/// # Arguments
///
/// * `ctx` - The context of the request, containing the accounts involved in the admin update.
/// * `args` - The TransferAdminArg struct containing the new admin's public key.
///
/// # Returns
///
/// Returns `Ok(())` if setting the pending admin is successful, otherwise returns an error.
#[access_control(TransferAdmin::validate(&ctx.accounts))]
pub fn transfer_admin(ctx: Context<TransferAdmin>, args: TransferAdminArgs) -> Result<()> {
    ctx.accounts.integrator_config.pending_admin = Some(args.new_admin);
    Ok(())
}

/// Claims the admin rights for an IntegratorConfig account.
///
/// This function allows only the pending admin to claim the admin rights,
/// completing the two-step admin transfer process.
///
/// # Arguments
///
/// * `ctx` - The context of the request, containing the accounts involved in claiming admin rights.
///
/// # Returns
///
/// Returns `Ok(())` if claiming admin rights is successful, otherwise returns an error.
pub fn claim_admin(ctx: Context<ClaimAdmin>) -> Result<()> {
    // The constraints in ClaimAdmin struct ensure that pending_admin is Some and matches the signer
    // or the admin matches the signer
    ctx.accounts.integrator_config.admin = Some(ctx.accounts.new_admin.key());
    ctx.accounts.integrator_config.pending_admin = None;
    Ok(())
}
