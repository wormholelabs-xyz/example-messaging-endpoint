use crate::error::EndpointError;
use crate::event::{RecvAdapterDisabledForChain, SendAdapterDisabledForChain};
use crate::instructions::common::AdapterInfoArgs;
use crate::state::{AdapterInfo, IntegratorChainConfig, IntegratorConfig};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: AdapterInfoArgs)]
pub struct DisableAdapter<'info> {
    /// The admin account that has the authority to disable adapters
    pub admin: Signer<'info>,

    /// The integrator config account
    /// The account constraints here make sure that the one signing this transaction is the admin
    /// of the config
    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program_id.as_ref()],
        bump = integrator_config.bump,
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

    /// The registered adapter account
    /// This makes sure that that the adapter is registered. Else, it will throw
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
}

impl<'info> DisableAdapter<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}

/// Disables a receive adapter
///
/// # Arguments
///
/// * `ctx` - The context of the request
/// * `_args` - The arguments for disabling the adapter (unused in this function)
///
/// # Returns
///
/// * `Result<()>` - Ok if the adapter was successfully disabled, otherwise an error
///
/// # Events
///
/// Emits a `RecvAdapterDisabledForChain` event
#[access_control(DisableAdapter::validate(&ctx.accounts))]
pub fn disable_recv_adapter(ctx: Context<DisableAdapter>, args: AdapterInfoArgs) -> Result<()> {
    let adapter_info = &ctx.accounts.adapter_info;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Check if the adapter is already disabled
    if !integrator_chain_config
        .recv_adapter_bitmap
        .get(adapter_info.index)?
    {
        return Err(EndpointError::AdapterAlreadyDisabled.into());
    }

    // Disable the adapter in the bitmap
    integrator_chain_config
        .recv_adapter_bitmap
        .set(adapter_info.index, false)?;

    emit_cpi!(RecvAdapterDisabledForChain {
        integrator: args.integrator_program_id,
        chain: args.chain_id,
        adapter: args.adapter_program_id,
    });

    Ok(())
}

/// Disables a send adapter
///
/// # Arguments
///
/// * `ctx` - The context of the request
/// * `_args` - The arguments for disabling the adapter (unused in this function)
///
/// # Returns
///
/// * `Result<()>` - Ok if the adapter was successfully disabled, otherwise an error
///
/// # Events
///
/// Emits a `SendAdapterDisabledForChain` event
#[access_control(DisableAdapter::validate(&ctx.accounts))]
pub fn disable_send_adapter(ctx: Context<DisableAdapter>, args: AdapterInfoArgs) -> Result<()> {
    let adapter_info = &ctx.accounts.adapter_info;
    let integrator_chain_config = &mut ctx.accounts.integrator_chain_config;

    // Check if the adapter is already disabled
    if !integrator_chain_config
        .send_adapter_bitmap
        .get(adapter_info.index)?
    {
        return Err(EndpointError::AdapterAlreadyDisabled.into());
    }

    // Disable the adapter in the bitmap
    integrator_chain_config
        .send_adapter_bitmap
        .set(adapter_info.index, false)?;

    emit_cpi!(SendAdapterDisabledForChain {
        integrator: args.integrator_program_id,
        chain: args.chain_id,
        adapter: args.adapter_program_id,
    });

    Ok(())
}
