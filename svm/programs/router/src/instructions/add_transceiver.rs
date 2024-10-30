use crate::state::{IntegratorConfig, TransceiverInfo};
use anchor_lang::prelude::*;

/// Arguments for the add_transceiver instruction
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddTransceiverArgs {
    /// The Pubkey of the integrator program
    pub integrator_program_id: Pubkey,

    /// The Pubkey of the transceiver to be registered
    pub transceiver_program_id: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: AddTransceiverArgs)]
pub struct AddTransceiver<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin registered on IntegratorConfig
    pub admin: Signer<'info>,

    /// The integrator config account
    /// This makes sure that the admin signing this ix is the one registered in the IntegratorConfig
    /// The new registered transceiver will be pushed to the `transceiver_infos` field in
    /// this account
    /// `has_one` constraint checks if admin signer is the current admin of the config
    #[account(
        mut,
        seeds = [IntegratorConfig::SEED_PREFIX, args.integrator_program_id.as_ref()],
        bump = integrator_config.bump,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The account to store information about the registered transceiver
    /// The `init` constraint checks that the transceiver has not been added. If it is,
    /// `AccountAlreadyInUse` error will be thrown
    #[account(
        init,
        payer = payer,
        space = 8 + TransceiverInfo::INIT_SPACE,
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.transceiver_program_id.as_ref(),
        ],
        bump
    )]
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// The system program
    pub system_program: Program<'info, System>,
}

impl<'info> AddTransceiver<'info> {
    pub fn validate(&self) -> Result<()> {
        self.integrator_config.check_admin(&self.admin)
    }
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
///     * `transceiver_program_id`: The Pubkey of the transceiver to be registered.
///
/// # Returns
///
/// Returns `Ok(())` if the transceiver is successfully registered, or an error otherwise.
#[access_control(AddTransceiver::validate(&ctx.accounts))]
pub fn add_transceiver(ctx: Context<AddTransceiver>, args: AddTransceiverArgs) -> Result<()> {
    let index = ctx.accounts.integrator_config.transceiver_infos.len() as u8;

    // Add the new transceiver to the list
    // The vector length check is in `add_transceiver`
    ctx.accounts
        .integrator_config
        .add_transceiver(args.transceiver_program_id)?;

    // Initialize TransceiverInfo
    ctx.accounts.transceiver_info.set_inner(TransceiverInfo {
        bump: ctx.bumps.transceiver_info,
        index,
        integrator_program_id: args.integrator_program_id,
        transceiver_program_id: args.transceiver_program_id,
    });

    Ok(())
}
