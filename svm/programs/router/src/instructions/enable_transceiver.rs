use crate::error::RouterError;
use crate::instructions::common::TransceiverInfoArgs;
use crate::state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
use crate::utils::bitmap::Bitmap;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(args: TransceiverInfoArgs)]
pub struct EnableTransceiver<'info> {
    /// The account that pays for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin account that has the authority to set transceivers
    pub admin: Signer<'info>,

    /// The integrator config account
    /// The account constraints here make sure that the one signing this transaction is the admin
    /// of the config
    /// The `has_one` constraint checks if admin signer is the current admin of the config
    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program_id.as_ref()],
        bump = integrator_config.bump,
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
            args.integrator_program_id.as_ref(),
            args.chain_id.to_be_bytes().as_ref(),
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
            args.integrator_program_id.as_ref(),
            args.transceiver_program_id.as_ref(),
        ],
        bump = registered_transceiver.bump,
    )]
    pub registered_transceiver: Account<'info, TransceiverInfo>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

impl<'info> EnableTransceiver<'info> {
    pub fn validate(&self, args: &TransceiverInfoArgs) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)?;

        // Ensure chain_id is not zero
        require!(args.chain_id != 0, RouterError::InvalidChainId);

        Ok(())
    }
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
#[access_control(EnableTransceiver::validate(&ctx.accounts, &args))]
pub fn enable_recv_transceiver(
    ctx: Context<EnableTransceiver>,
    args: TransceiverInfoArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // If chain_id is 0, this is initial setup
    if integrator_chain_config.chain_id == 0 {
        integrator_chain_config.set_inner(IntegratorChainConfig {
            chain_id: args.chain_id,
            bump: ctx.bumps.integrator_chain_config,
            integrator_program_id: args.integrator_program_id,
            send_transceiver_bitmap: Bitmap::new(),
            recv_transceiver_bitmap: Bitmap::new(),
        });
    }

    if integrator_chain_config
        .recv_transceiver_bitmap
        .get(registered_transceiver.index)?
    {
        return Err(RouterError::TransceiverAlreadyEnabled.into());
    }

    integrator_chain_config
        .recv_transceiver_bitmap
        .set(registered_transceiver.index, true)?;

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
#[access_control(EnableTransceiver::validate(&ctx.accounts, &args))]
pub fn enable_send_transceiver(
    ctx: Context<EnableTransceiver>,
    args: TransceiverInfoArgs,
) -> Result<()> {
    let registered_transceiver = &ctx.accounts.registered_transceiver;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // If chain_id is 0, this is initial setup
    if integrator_chain_config.chain_id == 0 {
        integrator_chain_config.set_inner(IntegratorChainConfig {
            chain_id: args.chain_id,
            bump: ctx.bumps.integrator_chain_config,
            integrator_program_id: args.integrator_program_id,
            send_transceiver_bitmap: Bitmap::new(),
            recv_transceiver_bitmap: Bitmap::new(),
        });
    }

    if integrator_chain_config
        .send_transceiver_bitmap
        .get(registered_transceiver.index)?
    {
        return Err(RouterError::TransceiverAlreadyEnabled.into());
    }

    integrator_chain_config
        .send_transceiver_bitmap
        .set(registered_transceiver.index, true)?;

    Ok(())
}
