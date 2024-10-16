use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterArgs {
    // Integrator Program
    pub integrator_program_id: Pubkey,

    // Bump to make sure the same PDA is derived
    pub integrator_program_pda_bump: u8,
}

#[derive(Accounts)]
#[instruction(args: RegisterArgs)]
pub struct Register<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin of the IntegratorConfig account
    /// CHECK: The integrator program is responsible for passing the correct admin
    pub admin: UncheckedAccount<'info>,

    /// The IntegratorConfig account being initialized
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

    /// The integrator program's PDA
    /// This makes sure that the Signer is a Integrator Program PDA Signer
    #[account(
        seeds = [b"router_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program_pda: Signer<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Register an integrator program with the router
///
/// This function initializes an IntegratorConfig account for the given integrator program.
/// It sets up the configuration with the provided admin and program ID, and initializes
/// an empty list of registered transceivers.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved
/// * `args` - The arguments for the register instruction, containing:
///   - `integrator_program_id`: The public key of the integrator program
///   - `integrator_program_pda_bump`: The bump used to derive the integrator program's PDA
///
/// # Returns
///
/// Returns `Ok(())` if the registration is successful, or an error if it fails
pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
    msg!(
        "Initializing IntegratorConfig for program: {}",
        args.integrator_program_id
    );

    // Initialize the IntegratorConfig account with the provided information
    ctx.accounts.integrator_config.set_inner(IntegratorConfig {
        bump: ctx.bumps.integrator_config,
        admin: ctx.accounts.admin.key(),
        integrator_program_id: args.integrator_program_id,
        registered_transceivers: Vec::new(),
    });

    msg!("IntegratorConfig initialized successfully");
    Ok(())
}
