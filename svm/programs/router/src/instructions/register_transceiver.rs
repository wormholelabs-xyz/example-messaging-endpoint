use crate::error::RouterError;
use crate::state::{IntegratorConfig, RegisteredTransceiver};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, integrator_program.key().as_ref()],
        bump = integrator_config.bump,
        has_one = owner @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + RegisteredTransceiver::INIT_SPACE,
        seeds = [
            RegisteredTransceiver::SEED_PREFIX,
            integrator_program.key().as_ref(),
            &[integrator_config.next_transceiver_id],
        ],
        bump
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    transceiver_address: Pubkey,
) -> Result<()> {
    let transceiver_id = ctx.accounts.integrator_config.next_transceiver_id;

    // Check if we've reached the maximum number of transceivers
    if transceiver_id >= IntegratorConfig::MAX_TRANSCEIVERS {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Increment next_transceiver_id
    // Note: We don't need to test for reinitialization of the registered_transceiver account
    // because the seed `next_transceiver_id` is auto-incremented, ensuring a unique PDA for each call.
    ctx.accounts.integrator_config.next_transceiver_id = transceiver_id.checked_add(1).unwrap();

    // Initialize RegisteredTransceiver
    ctx.accounts
        .registered_transceiver
        .set_inner(RegisteredTransceiver {
            bump: ctx.bumps.registered_transceiver,
            id: transceiver_id,
            integrator_program_id: ctx.accounts.integrator_program.key(),
            address: transceiver_address,
        });

    Ok(())
}
