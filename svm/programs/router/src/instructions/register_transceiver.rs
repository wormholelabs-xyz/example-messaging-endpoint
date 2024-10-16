use crate::error::RouterError;
use crate::state::{IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

/// Arguments for the register_transceiver instruction
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterTransceiverArgs {
    /// The Pubkey of the integrator program
    pub integrator_program: Pubkey,

    /// The Pubkey of the transceiver to be registered
    pub transceiver_address: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: RegisterTransceiverArgs)]
pub struct RegisterTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin registered on IntegratroConfig
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The integrator config account
    /// This makes sure that the admin signing this ix is the one registed in the IntegratorConfig
    /// The new registered transceiver will be pushed to the `registered_transceivers` field in
    /// this account
    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program.as_ref()],
        bump = integrator_config.bump,
        has_one = admin @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The account to store information about the registered transceiver
    #[account(
        init,
        payer = payer,
        space = 8 + TransceiverInfo::INIT_SPACE,
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            args.integrator_program.as_ref(),
            args.transceiver_address.as_ref(),
        ],
        bump
    )]
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// The system program
    pub system_program: Program<'info, System>,
}

/// Register a new transceiver for an integrator.
///
/// This function performs the following steps:
/// 1. Checks if the maximum number of transceivers has been reached.
/// 2. Adds the new transceiver to the list of registered transceivers in IntegratorConfig
/// 3. Initializes the TransceiverInfo account with the provided information.
///
/// # Arguments
///
/// * `ctx` - The context for the instruction, containing the accounts.
/// * `args` - The arguments for registering a transceiver, including:
///     * `integrator_program`: The Pubkey of the integrator program.
///     * `transceiver_address`: The Pubkey of the transceiver to be registered.
///
/// # Returns
///
/// Returns `Ok(())` if the transceiver is successfully registered, or an error otherwise.
pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    args: RegisterTransceiverArgs,
) -> Result<()> {
    let transceiver_id = ctx.accounts.integrator_config.registered_transceivers.len() as u8;

    // Check if we've reached the maximum number of transceivers
    if transceiver_id >= IntegratorConfig::MAX_TRANSCEIVERS as u8 {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Add the new transceiver to the list
    ctx.accounts
        .integrator_config
        .registered_transceivers
        .push(args.transceiver_address);

    // Initialize TransceiverInfo
    ctx.accounts.transceiver_info.set_inner(TransceiverInfo {
        bump: ctx.bumps.transceiver_info,
        id: transceiver_id,
        integrator_program_id: args.integrator_program,
        transceiver_address: args.transceiver_address,
    });

    Ok(())
}
