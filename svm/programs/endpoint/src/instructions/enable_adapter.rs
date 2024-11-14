use crate::error::EndpointError;
use crate::event::{RecvAdapterEnabledForChain, SendAdapterEnabledForChain};
use crate::instructions::common::AdapterInfoArgs;
use crate::state::{AdapterInfo, IntegratorChainConfig, IntegratorConfig};
use crate::utils::bitmap::Bitmap;
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: AdapterInfoArgs)]
pub struct EnableAdapter<'info> {
    /// The account that pays for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin account that has the authority to set adapters
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

    /// The registered adapter account
    /// This makes sure that the adapter is registered. Else, it will throw
    /// `AccountNotInitialized`
    #[account(
        seeds = [
            AdapterInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.adapter_program_id.as_ref(),
        ],
        bump = adapter_info.bump,
    )]
    pub adapter_info: Account<'info, AdapterInfo>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

impl<'info> EnableAdapter<'info> {
    pub fn validate(&self, args: &AdapterInfoArgs) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)?;

        // Ensure chain_id is not zero
        require!(args.chain_id != 0, EndpointError::InvalidChainId);

        Ok(())
    }
}

/// Enables a receive adapter for the integrator chain configuration
///
/// This function performs the following steps:
/// 1. Initializes the IntegratorChainConfig if it's not already set up.
/// 2. Checks if the adapter is already enabled.
/// 3. Enables the adapter in the receive adapter bitmap.
/// 4. Emits a RecvAdapterEnabledForChain event.
///
/// # Arguments
///
/// * `ctx` - The context of accounts
/// * `_args` - The arguments for setting the adapter
///   * `chain_id` - The chain ID for the integrator chain configuration
///   * `adapter` - The public key of the adapter to be set
///   * `integrator_program` - The public key of the integrator program
///
/// # Returns
///
/// * `Result<()>` - Ok if the adapter was successfully enabled, otherwise an error
///
/// # Errors
///
/// This function will return an error if:
/// * The adapter is already enabled (EndpointError::AdapterAlreadyEnabled)
///
/// # Events
///
/// Emits a `RecvAdapterEnabledForChain` event
#[access_control(EnableAdapter::validate(&ctx.accounts, &args))]
pub fn enable_recv_adapter(ctx: Context<EnableAdapter>, args: AdapterInfoArgs) -> Result<()> {
    let adapter_info = &ctx.accounts.adapter_info;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // If chain_id is 0, this is initial setup
    if integrator_chain_config.chain_id == 0 {
        integrator_chain_config.set_inner(IntegratorChainConfig {
            chain_id: args.chain_id,
            bump: ctx.bumps.integrator_chain_config,
            integrator_program_id: args.integrator_program_id,
            send_adapter_bitmap: Bitmap::new(),
            recv_adapter_bitmap: Bitmap::new(),
        });
    }

    if integrator_chain_config
        .recv_adapter_bitmap
        .get(adapter_info.index)?
    {
        return Err(EndpointError::AdapterAlreadyEnabled.into());
    }

    integrator_chain_config
        .recv_adapter_bitmap
        .set(adapter_info.index, true)?;

    emit_cpi!(RecvAdapterEnabledForChain {
        integrator: args.integrator_program_id,
        chain: args.chain_id,
        adapter: args.adapter_program_id,
    });

    Ok(())
}

/// Enables a send adapter for the integrator chain configuration
///
/// This function performs the following steps:
/// 1. Initializes the IntegratorChainConfig if it's not already set up.
/// 2. Checks if the adapter is already enabled.
/// 3. Enables the adapter in the send adapter bitmap.
/// 4. Emits a SendAdapterEnabledForChain event.
///
/// # Arguments
///
/// * `ctx` - The context of accounts
/// * `_args` - The arguments for setting the adapter
///   * `chain_id` - The chain ID for the integrator chain configuration
///   * `adapter` - The public key of the adapter to be set
///   * `integrator_program` - The public key of the integrator program
///
/// # Returns
///
/// * `Result<()>` - Ok if the adapter was successfully enabled, otherwise an error
///
/// # Errors
///
/// This function will return an error if:
/// * The adapter is already enabled (EndpointError::AdapterAlreadyEnabled)
///
/// # Events
///
/// Emits a `SendAdapterEnabledForChain` event
#[access_control(EnableAdapter::validate(&ctx.accounts, &args))]
pub fn enable_send_adapter(ctx: Context<EnableAdapter>, args: AdapterInfoArgs) -> Result<()> {
    let adapter_info = &ctx.accounts.adapter_info;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // If chain_id is 0, this is initial setup
    if integrator_chain_config.chain_id == 0 {
        integrator_chain_config.set_inner(IntegratorChainConfig {
            chain_id: args.chain_id,
            bump: ctx.bumps.integrator_chain_config,
            integrator_program_id: args.integrator_program_id,
            send_adapter_bitmap: Bitmap::new(),
            recv_adapter_bitmap: Bitmap::new(),
        });
    }

    if integrator_chain_config
        .send_adapter_bitmap
        .get(adapter_info.index)?
    {
        return Err(EndpointError::AdapterAlreadyEnabled.into());
    }

    integrator_chain_config
        .send_adapter_bitmap
        .set(adapter_info.index, true)?;

    emit_cpi!(SendAdapterEnabledForChain {
        integrator: args.integrator_program_id,
        chain: args.chain_id,
        adapter: args.adapter_program_id,
    });

    Ok(())
}
