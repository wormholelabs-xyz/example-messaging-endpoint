use crate::error::RouterError;
use crate::state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetTransceiverArgs {
    /// The chain ID for the integrator chain configuration
    pub chain_id: u16,

    /// The Pubkey of the transceiver to be set
    pub transceiver: Pubkey,

    /// The Pubkey of the integrator program
    pub integrator_program: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: SetTransceiverArgs)]
pub struct SetTransceiver<'info> {
    /// The account that pays for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin account that has the authority to set transceivers
    pub admin: Signer<'info>,

    /// The integrator config account
    /// The account constraints here make sure that the one signing this transaction is the admin
    /// of the config
    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program.as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The integrator chain config account
    /// This account will be initialized if it doesn't exist, and its bitmap will be updated
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

    /// The registered transceiver account
    /// This makes sure that the transceiver is registered. Else, it will throw
    /// `AccountNotInitialized`
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

/// Sets a receive transceiver for the integrator chain configuration
///
/// # Arguments
///
/// * `ctx` - The context of accounts
/// * `_args` - The arguments for setting the transceiver
///   * `chain_id` - The chain ID for the integrator chain configuration
///   * `transceiver` - The public key of the transceiver to be set
///   * `integrator_program` - The public key of the integrator program
///
/// # Returns
///
/// * `Result<()>` - The result of the operation
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

/// Sets a send transceiver for the integrator chain configuration
///
/// # Arguments
///
/// * `ctx` - The context of accounts
/// * `_args` - The arguments for setting the transceiver
///   * `chain_id` - The chain ID for the integrator chain configuration
///   * `transceiver` - The public key of the transceiver to be set
///   * `integrator_program` - The public key of the integrator program
///
/// # Returns
///
/// * `Result<()>` - The result of the operation
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
