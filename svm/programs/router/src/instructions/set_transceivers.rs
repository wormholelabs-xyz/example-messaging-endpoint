use crate::error::RouterError;
use crate::state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetTransceiverArgs {
    pub chain_id: u16,
    pub transceiver: Pubkey,
    pub integrator_program: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: SetTransceiverArgs)]
pub struct SetTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub admin: Signer<'info>,

    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program.as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + IntegratorChainConfig::INIT_SPACE,
        seeds = [
            IntegratorChainConfig::SEED_PREFIX,
            args.integrator_program.as_ref(),
            args.chain_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub integrator_chain_config: Account<'info, IntegratorChainConfig>,

    #[account(
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            args.integrator_program.as_ref(),
            args.transceiver.as_ref(),
        ],
        bump = registered_transceiver.bump,
    )]
    pub registered_transceiver: Account<'info, TransceiverInfo>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

pub fn set_recv_transceiver(ctx: Context<SetTransceiver>, _args: SetTransceiverArgs) -> Result<()> {
    msg!(
        "Set Recv Transceiver PDA: {:?}",
        ctx.accounts.integrator_chain_config.key()
    );

    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    if integrator_chain_config
        .recv_transceiver_bitmap
        .get(registered_transceiver.id)?
    {
        return Err(RouterError::TransceiverAlreadyEnabled.into());
    }

    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.id, true)?;

    Ok(())
}

pub fn set_send_transceiver(ctx: Context<SetTransceiver>, _args: SetTransceiverArgs) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    if integrator_chain_config
        .send_transceiver_bitmap
        .get(registered_transceiver.id)?
    {
        return Err(RouterError::TransceiverAlreadyEnabled.into());
    }

    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.id, true)?;

    Ok(())
}
