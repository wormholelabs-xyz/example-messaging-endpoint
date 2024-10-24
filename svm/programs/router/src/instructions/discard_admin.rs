use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

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

#[access_control(DiscardAdmin::validate(&ctx.accounts))]
pub fn discard_admin(ctx: Context<DiscardAdmin>) -> Result<()> {
    ctx.accounts.integrator_config.admin = None;
    Ok(())
}
