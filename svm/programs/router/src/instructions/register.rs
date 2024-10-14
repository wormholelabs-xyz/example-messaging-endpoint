use crate::state::IntegratorConfig;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterArgs {
    pub integrator_program_id: Pubkey,
}

/// Accounts struct for initializing an IntegratorConfig account
#[derive(Accounts)]
#[instruction(args: RegisterArgs)]
pub struct Register<'info> {
    /// The account paying for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The admin of the IntegratorConfig account
    /// TODO: check if this should be a signer
    /// CHECK: The integrator program is responsible for passing the correct owner
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
    #[account(
        seeds = [b"router_integrator", args.integrator_program_id.as_ref()],
        bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program: SystemAccount<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
    msg!(
        "Initializing IntegratorConfig for program: {}",
        args.integrator_program_id
    );

    ctx.accounts.integrator_config.set_inner(IntegratorConfig {
        bump: ctx.bumps.integrator_config,
        admin: ctx.accounts.admin.key(),
        integrator_program_id: args.integrator_program_id,
        registered_transceivers: Vec::new(),
    });

    msg!("IntegratorConfig initialized successfully");
    Ok(())
}
