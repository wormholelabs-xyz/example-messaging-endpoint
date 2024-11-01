use crate::error::EndpointError;
use crate::event::{AdminUpdateRequested, AdminUpdated};
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferAdminArgs {
    /// The new_admin to be assigned
    pub new_admin: Pubkey,

    /// The integrator_program for the integrator_config
    pub integrator_program_id: Pubkey,
}

#[event_cpi]
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

#[event_cpi]
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
/// This function performs the following steps:
/// 1. Validates that the current admin is initiating the transfer.
/// 2. Sets a pending admin for the IntegratorConfig account.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved
/// * `args` - The arguments for the transfer_admin instruction, containing:
///   - `new_admin`: The public key of the new admin
///   - `integrator_program_id`: The public key of the integrator program
///
/// # Returns
///
/// Returns `Ok(())` if setting the pending admin is successful, or an error if it fails
///
/// # Errors
///
/// This function will return an error if:
/// * There is already a transfer in progress (EndpointError::AdminTransferInProgress)
/// * The current admin is not the signer (EndpointError::CallerNotAuthorized)
///
/// # Events
/// Emits an `AdminUpdateRequested` event
#[access_control(TransferAdmin::validate(&ctx.accounts))]
pub fn transfer_admin(ctx: Context<TransferAdmin>, args: TransferAdminArgs) -> Result<()> {
    ctx.accounts.integrator_config.pending_admin = Some(args.new_admin);

    emit_cpi!(AdminUpdateRequested {
        integrator: args.integrator_program_id,
        old_admin: ctx.accounts.admin.key(),
        new_admin: args.new_admin,
    });

    Ok(())
}

/// Claims the admin rights for an IntegratorConfig account.
///
/// This function performs the following steps:
/// 1. Validates that the signer is either the pending admin or the current admin.
/// 2. Sets the new admin as the current admin.
/// 3. Clears the pending admin field.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved
///
/// # Returns
///
/// Returns `Ok(())` if claiming admin rights is successful, or an error if it fails
///
/// # Errors
///
/// This function will return an error if:
/// * There is no pending admin transfer (EndpointError::NoAdminTransferInProgress)
/// * The signer is not the pending admin or the current admin (EndpointError::CallerNotAuthorized)
///
/// Emits an `AdminUpdated` event
pub fn claim_admin(ctx: Context<ClaimAdmin>) -> Result<()> {
    // The constraints in ClaimAdmin struct ensure that pending_admin is Some and matches the signer
    // or the admin matches the signer
    ctx.accounts.integrator_config.admin = Some(ctx.accounts.new_admin.key());
    ctx.accounts.integrator_config.pending_admin = None;

    emit_cpi!(AdminUpdated {
        integrator: ctx.accounts.integrator_config.integrator_program_id,
        old_admin: ctx.accounts.integrator_config.admin.unwrap(),
        new_admin: ctx.accounts.new_admin.key(),
    });

    Ok(())
}
