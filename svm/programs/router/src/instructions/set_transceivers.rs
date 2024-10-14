use crate::error::RouterError;
use crate::state::{IntegratorChainConfig, IntegratorConfig, RegisteredTransceiver};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetTransceiverArgs {
    pub chain_id: u16,
}

#[derive(Accounts)]
#[instruction(args: SetTransceiverArgs)]
pub struct SetTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub admin: Signer<'info>,

    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, integrator_program.key().as_ref()],
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
            integrator_program.key().as_ref(),
            args.chain_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub integrator_chain_config: Account<'info, IntegratorChainConfig>,

    #[account(
        seeds = [
            RegisteredTransceiver::SEED_PREFIX,
            integrator_program.key().as_ref(),
            transceiver.key().as_ref(),
        ],
        bump = registered_transceiver.bump,
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,

    /// The transceiver account being set
    /// CHECK: This account is only used as a reference for PDA derivation and is not accessed directly
    pub transceiver: AccountInfo<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

pub fn set_recv_transceiver(ctx: Context<SetTransceiver>, _args: SetTransceiverArgs) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Set the bit corresponding to the registered_transceiver id
    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.id, true)?;

    Ok(())
}

//TODO: Disable shouldn't init chain config
pub fn disable_recv_transceiver(
    ctx: Context<SetTransceiver>,
    _args: SetTransceiverArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Clear the bit corresponding to the registered_transceiver id
    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.id, false)?;

    Ok(())
}

pub fn set_send_transceiver(ctx: Context<SetTransceiver>, _args: SetTransceiverArgs) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Set the bit corresponding to the registered_transceiver id
    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.id, true)?;

    Ok(())
}

pub fn disable_send_transceiver(
    ctx: Context<SetTransceiver>,
    _args: SetTransceiverArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Clear the bit corresponding to the registered_transceiver id
    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.id, false)?;

    Ok(())
}
