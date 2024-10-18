use crate::error::RouterError;
use crate::instructions::common::TransceiverInfoArgs;
use crate::state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(args: TransceiverInfoArgs)]
pub struct DisableTransceiver<'info> {
    /// The admin account that has the authority to disable transceivers
    pub admin: Signer<'info>,

    /// The integrator config account
    /// The account constraints here make sure that the one signing this transaction is the admin
    /// of the config
    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program_id.as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::CallerNotAuthorized,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The integrator chain config account
    /// The bitmap of in this chain config account will be updated
    #[account(
        mut,
        seeds = [
            IntegratorChainConfig::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.chain_id.to_be_bytes().as_ref(),
        ],
        bump,
    )]
    pub integrator_chain_config: Account<'info, IntegratorChainConfig>,

    /// The registered transceiver account
    /// This makes sure that that the transceiver is registered. Else, it will throw
    /// `AccountNotInitialized`
    #[account(
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.transceiver_program_id.as_ref(),
        ],
        bump = registered_transceiver.bump,
    )]
    pub registered_transceiver: Account<'info, TransceiverInfo>,
}
impl<'info> DisableTransceiver<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}
/// Disables a receive transceiver
///
/// # Arguments
///
/// * `ctx` - The context of the request
/// * `_args` - The arguments for disabling the transceiver (unused in this function)
///
/// # Returns
///
/// * `Result<()>` - Ok if the transceiver was successfully disabled, otherwise an error
#[access_control(DisableTransceiver::validate(&ctx.accounts))]
pub fn disable_recv_transceiver(
    ctx: Context<DisableTransceiver>,
    _args: TransceiverInfoArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Check if the transceiver is already disabled
    if !integrator_chain_config
        .recv_transceiver_bitmap
        .get(registered_transceiver.index)?
    {
        return Err(RouterError::TransceiverAlreadyDisabled.into());
    }

    // Disable the transceiver in the bitmap
    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.index, false)?;

    Ok(())
}

/// Disables a send transceiver
///
/// # Arguments
///
/// * `ctx` - The context of the request
/// * `_args` - The arguments for disabling the transceiver (unused in this function)
///
/// # Returns
///
/// * `Result<()>` - Ok if the transceiver was successfully disabled, otherwise an error
#[access_control(DisableTransceiver::validate(&ctx.accounts))]
pub fn disable_send_transceiver(
    ctx: Context<DisableTransceiver>,
    _args: TransceiverInfoArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Check if the transceiver is already disabled
    if !integrator_chain_config
        .send_transceiver_bitmap
        .get(registered_transceiver.index)?
    {
        return Err(RouterError::TransceiverAlreadyDisabled.into());
    }

    // Disable the transceiver in the bitmap
    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.index, false)?;

    Ok(())
}
