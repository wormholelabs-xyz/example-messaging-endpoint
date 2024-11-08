use crate::event::AdminUpdated;
use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateAdminArgs {
    /// The new_admin to be assigned
    pub new_admin: Pubkey,

    /// The integrator_program for the integrator_config
    pub integrator_program_id: Pubkey,
}

#[event_cpi]
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
/// This function performs the following steps:
/// 1. Validates that the current admin is initiating the update.
/// 2. Checks if there's a pending admin transfer.
/// 3. Updates the admin field in the IntegratorConfig account.
/// 4. Emits an AdminUpdated event.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved
/// * `args` - The arguments for the update_admin instruction, containing:
///   - `new_admin`: The public key of the new admin
///   - `integrator_program_id`: The public key of the integrator program
///
/// # Returns
///
/// Returns `Ok(())` if the admin update is successful, or an error if it fails
///
/// # Errors
///
/// This function will return an error if:
/// * There is a pending admin transfer (EndpointError::AdminTransferInProgress)
/// * The current admin is not the signer (EndpointError::CallerNotAuthorized)
///
/// # Events
///
/// Emits an `AdminUpdated` event
#[access_control(UpdateAdmin::validate(&ctx.accounts))]
pub fn update_admin(ctx: Context<UpdateAdmin>, args: UpdateAdminArgs) -> Result<()> {
    ctx.accounts.integrator_config.admin = Some(args.new_admin);

    // Emit the AdminUpdated event
    emit_cpi!(AdminUpdated {
        integrator: args.integrator_program_id,
        new_admin: args.new_admin,
        old_admin: ctx.accounts.integrator_config.admin.unwrap(),
    });

    Ok(())
}
