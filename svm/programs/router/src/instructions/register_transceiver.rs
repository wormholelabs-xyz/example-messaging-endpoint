use crate::error::RouterError;
use crate::state::{IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, integrator_program.key().as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + TransceiverInfo::INIT_SPACE,
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            integrator_program.key().as_ref(),
            transceiver_address.key().as_ref(),
        ],
        bump
    )]
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,

    /// CHECK: This is the address of the transceiver being registered
    pub transceiver_address: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(ctx: Context<RegisterTransceiver>) -> Result<()> {
    let transceiver_id = ctx.accounts.integrator_config.registered_transceivers.len() as u8;

    // Check if we've reached the maximum number of transceivers
    if transceiver_id >= IntegratorConfig::MAX_TRANSCEIVERS as u8 {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Add the new transceiver to the list
    ctx.accounts
        .integrator_config
        .registered_transceivers
        .push(ctx.accounts.transceiver_address.key());

    // Initialize TransceiverInfo
    ctx.accounts.transceiver_info.set_inner(TransceiverInfo {
        bump: ctx.bumps.transceiver_info,
        id: transceiver_id,
        integrator_program_id: ctx.accounts.integrator_program.key(),
        transceiver_address: ctx.accounts.transceiver_address.key(),
    });

    Ok(())
}
