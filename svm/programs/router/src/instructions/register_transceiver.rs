use crate::error::RouterError;
use crate::state::{IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterTransceiverArgs {
    pub integrator_program: Pubkey,
    pub transceiver_address: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: RegisterTransceiverArgs)]
pub struct RegisterTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program.as_ref()],
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
            args.integrator_program.as_ref(),
            args.transceiver_address.as_ref(),
        ],
        bump
    )]
    pub transceiver_info: Account<'info, TransceiverInfo>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    args: RegisterTransceiverArgs,
) -> Result<()> {
    let transceiver_id = ctx.accounts.integrator_config.registered_transceivers.len() as u8;

    // Check if we've reached the maximum number of transceivers
    if transceiver_id >= IntegratorConfig::MAX_TRANSCEIVERS as u8 {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Add the new transceiver to the list
    ctx.accounts
        .integrator_config
        .registered_transceivers
        .push(args.transceiver_address);

    // Initialize TransceiverInfo
    ctx.accounts.transceiver_info.set_inner(TransceiverInfo {
        bump: ctx.bumps.transceiver_info,
        id: transceiver_id,
        integrator_program_id: args.integrator_program,
        transceiver_address: args.transceiver_address,
    });

    Ok(())
}
