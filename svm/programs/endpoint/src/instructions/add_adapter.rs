use crate::{
    event::AdapterAdded,
    state::{AdapterInfo, IntegratorConfig},
};
use anchor_lang::prelude::*;

/// Arguments for the add_adapter instruction
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddAdapterArgs {
    /// The Pubkey of the integrator program
    pub integrator_program_id: Pubkey,

    /// The Pubkey of the adapter to be registered
    pub adapter_program_id: Pubkey,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: AddAdapterArgs)]
pub struct AddAdapter<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin registered on IntegratorConfig
    pub admin: Signer<'info>,

    /// The integrator config account
    /// This makes sure that the admin signing this ix is the one registered in the IntegratorConfig
    /// The new registered adapter will be pushed to the `adapter_infos` field in
    /// this account
    /// `has_one` constraint checks if admin signer is the current admin of the config
    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program_id.as_ref()],
        bump = integrator_config.bump,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The account to store information about the registered adapter
    /// The `init` constraint checks that the adapter has not been added. If it is,
    /// `AccountAlreadyInUse` error will be thrown
    #[account(
        init,
        payer = payer,
        space = 8 + AdapterInfo::INIT_SPACE,
        seeds = [
            AdapterInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.adapter_program_id.as_ref(),
        ],
        bump
    )]
    pub adapter_info: Account<'info, AdapterInfo>,

    /// The system program
    pub system_program: Program<'info, System>,
}

impl<'info> AddAdapter<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
}

/// Register a new adapter for an integrator.
///
/// This function performs the following steps:
/// 1. Checks if the maximum number of adapters has been reached.
/// 2. Adds the new adapter to the list of registered adapters in IntegratorConfig
/// 3. Initializes the AdapterInfo account with the provided information.
///
/// # Arguments
///
/// * `ctx` - The context for the instruction, containing the accounts.
/// * `args` - The arguments for registering a adapter, including:
///     * `integrator_program`: The Pubkey of the integrator program.
///     * `adapter_program_id`: The Pubkey of the adapter to be registered.
///
/// # Returns
///
/// Returns `Ok(())` if the adapter is successfully registered, or an error otherwise.
///
/// # Events
///
/// Emits a `AdapterAdded` event
#[access_control(AddAdapter::validate(&ctx.accounts))]
pub fn add_adapter(ctx: Context<AddAdapter>, args: AddAdapterArgs) -> Result<()> {
    let index = ctx.accounts.integrator_config.adapter_infos.len() as u8;

    // Add the new adapter to the list
    // The vector length check is in `add_adapter`
    ctx.accounts
        .integrator_config
        .add_adapter(args.adapter_program_id)?;

    // Initialize AdapterInfo
    ctx.accounts.adapter_info.set_inner(AdapterInfo {
        bump: ctx.bumps.adapter_info,
        index,
        integrator_program_id: args.integrator_program_id,
        adapter_program_id: args.adapter_program_id,
    });

    emit_cpi!(AdapterAdded {
        integrator: args.integrator_program_id,
        adapter: args.adapter_program_id,
        adapters_num: index,
    });

    Ok(())
}
