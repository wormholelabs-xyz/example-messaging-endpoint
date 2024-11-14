use crate::{event::AdminDiscarded, state::IntegratorConfig};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
pub struct DiscardAdmin<'info> {
    /// The current admin of the IntegratorConfig account
    pub admin: Signer<'info>,

    /// The IntegratorConfig account being modified
    #[account(
        mut,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            integrator_config.integrator_program_id.key().as_ref(),
        ],
        bump = integrator_config.bump,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,
}

impl<'info> DiscardAdmin<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}

/// Discards the admin for an integrator configuration
///
/// This function removes the admin from the IntegratorConfig account,
/// effectively leaving the integrator without an admin. Only an admin is
/// authorized to do that.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
///
/// # Returns
///
/// Returns `Ok(())` if the admin is successfully discarded
///
///
/// # Events
///
/// Emits an `AdminDiscarded` event with the integrator's public key.
#[access_control(DiscardAdmin::validate(&ctx.accounts))]
pub fn discard_admin(ctx: Context<DiscardAdmin>) -> Result<()> {
    ctx.accounts.integrator_config.admin = None;

    emit_cpi!(AdminDiscarded {
        integrator: ctx.accounts.integrator_config.integrator_program_id,
    });

    Ok(())
}
