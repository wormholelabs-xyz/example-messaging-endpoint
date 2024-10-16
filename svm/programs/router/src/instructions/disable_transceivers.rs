use crate::error::RouterError;
use crate::state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DisableTransceiverArgs {
    pub chain_id: u16,
    pub transceiver: Pubkey,
    pub integrator_program: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: DisableTransceiverArgs)]
pub struct DisableTransceiver<'info> {
    pub admin: Signer<'info>,

    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program.as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    #[account(
        mut,
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
}

pub fn disable_recv_transceiver(
    ctx: Context<DisableTransceiver>,
    _args: DisableTransceiverArgs,
) -> Result<()> {
    msg!(
        "Disable Recv Transceiver PDA: {:?}",
        ctx.accounts.integrator_chain_config.key()
    );

    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    if !integrator_chain_config
        .recv_transceiver_bitmap
        .get(registered_transceiver.id)?
    {
        return Err(RouterError::TransceiverAlreadyDisabled.into());
    }

    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.id, false)?;

    Ok(())
}

pub fn disable_send_transceiver(
    ctx: Context<DisableTransceiver>,
    _args: DisableTransceiverArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    if !integrator_chain_config
        .send_transceiver_bitmap
        .get(registered_transceiver.id)?
    {
        return Err(RouterError::TransceiverAlreadyDisabled.into());
    }

    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.id, false)?;

    Ok(())
}
