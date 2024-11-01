use crate::{
    event::IntegratorRegistered,
    state::{IntegratorConfig, SequenceTracker},
};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterArgs {
    // Integrator Program
    pub integrator_program_id: Pubkey,

    // Bump to make sure the same PDA is derived
    pub integrator_program_pda_bump: u8,

    // Admin of the IntegratorConfig account
    pub admin: Pubkey,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: RegisterArgs)]
pub struct Register<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The IntegratorConfig account being initialized
    /// `init` constraint checks that caller is not already registered
    #[account(
        init,
        payer = payer,
        space = 8 + IntegratorConfig::INIT_SPACE,
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
        ],
        bump
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The SequenceTracker account being initialized
    #[account(
        init,
        payer = payer,
        space = 8 + SequenceTracker::INIT_SPACE,
        seeds = [
            SequenceTracker::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
        ],
        bump
    )]
    pub sequence_tracker: Account<'info, SequenceTracker>,

    /// The integrator program's PDA
    /// This makes sure that the Signer is a Integrator Program PDA Signer
    /// TODO: Ideally there is a `AccountUncheckedOwner` that does not explicitly enforce owner
    /// check on AccountUncheckedOwner<T> and use the `owner = another_program.ID` but it is not
    /// possible for now. So we have to pass in the bump manually in the args to use it here
    /// This is easier for monitoring anyways since you don't have to lookup the this account to
    /// get the integrator program id and bump
    /// Link to discussion: https://github.com/coral-xyz/anchor/issues/3285#issuecomment-2381329832
    #[account(
        seeds = [b"endpoint_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program_pda: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Register an integrator program with the endpoint
///
/// This function performs the following steps:
/// 1. Initializes an IntegratorConfig account for the given integrator program.
/// 2. Sets up the configuration with the provided admin and program ID.
/// 3. Initializes an empty list of registered adapters.
/// 4. Initializes a SequenceTracker account for the integrator program.
/// 5. Emits IntegratorRegistered events.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved
/// * `args` - The arguments for the register instruction, containing:
///   - `integrator_program_id`: The public key of the integrator program
///   - `integrator_program_pda_bump`: The bump used to derive the integrator program's PDA
///   - `admin`: The public key of the admin for the IntegratorConfig
///
/// # Returns
///
/// Returns `Ok(())` if the registration is successful, or an error if it fails
///
/// # Events
///
/// Emits `IntegratorRegistered` event
pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
    // Initialize the IntegratorConfig account with the provided information
    ctx.accounts.integrator_config.set_inner(IntegratorConfig {
        bump: ctx.bumps.integrator_config,
        admin: Some(args.admin),
        pending_admin: None,
        integrator_program_id: args.integrator_program_id,
        adapter_infos: Vec::new(),
    });

    // Initialize the SequenceTracker account with default values
    ctx.accounts.sequence_tracker.set_inner(SequenceTracker {
        bump: ctx.bumps.sequence_tracker,
        integrator_program_id: args.integrator_program_id,
        sequence: 0,
    });

    emit_cpi!(IntegratorRegistered {
        integrator: args.integrator_program_id,
        admin: args.admin,
    });

    emit!(IntegratorRegistered {
        integrator: args.integrator_program_id,
        admin: args.admin,
    });

    Ok(())
}
