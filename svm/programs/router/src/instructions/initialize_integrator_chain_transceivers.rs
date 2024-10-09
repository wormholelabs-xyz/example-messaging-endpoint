use crate::{
    error::RouterError,
    state::{IntegratorChainTransceivers, IntegratorConfig},
};
use anchor_lang::prelude::*;

/// Accounts struct for initializing an IntegratorChainTransceivers account
#[derive(Accounts)]
#[instruction(chain_id: u16)]
pub struct InitializeIntegratorChainTransceivers<'info> {
    /// The owner of the IntegratorConfig account
    pub owner: Signer<'info>,

    /// The account paying for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The IntegratorChainTransceivers account being initialized
    #[account(
        init,
        payer = payer,
        space = 8 + IntegratorChainTransceivers::INIT_SPACE,
        seeds = [
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program.key().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,

    /// The integrator program
    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,

    /// The IntegratorConfig account
    #[account(
        seeds = [
            IntegratorConfig::SEED_PREFIX,
            integrator_program.key().as_ref(),
        ],
        bump,
        has_one = owner @ RouterError::InvalidIntegratorAuthority
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

pub fn initialize_integrator_chain_transceivers(
    ctx: Context<InitializeIntegratorChainTransceivers>,
    chain_id: u16,
) -> Result<()> {
    msg!(
        "Initializing IntegratorChainTransceivers for chain_id: {}",
        chain_id
    );

    ctx.accounts
        .integrator_chain_transceivers
        .set_inner(IntegratorChainTransceivers::new(
            ctx.bumps.integrator_chain_transceivers,
            chain_id,
            ctx.accounts.integrator_program.key(),
        ));

    msg!("IntegratorChainTransceivers initialized successfully");

    Ok(())
}
