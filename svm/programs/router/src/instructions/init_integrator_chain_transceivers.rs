use crate::{
    state::{Config, IntegratorChainTransceivers},
    utils::bitmap::Bitmap,
};
use anchor_lang::prelude::*;

/// Accounts struct for initializing an IntegratorChainTransceivers account
#[derive(Accounts)]
#[instruction(chain_id: u16)]
pub struct InitIntegratorChainTransceivers<'info> {
    /// The global configuration account
    #[account(
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// The owner of the IntegratorChainTransceivers account
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

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Initializes an IntegratorChainTransceivers account for a specific integrator_program and chain
///
/// This function sets up the initial state for managing transceivers on a particular chain
/// for a given integrator_program. It initializes counters and bitmaps for both incoming and outgoing
/// transceivers.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `chain_id` - The ID of the chain for which the transceivers are being initialized
///
/// # Returns
///
/// Returns `Ok(())` if the initialization is successful
pub fn init_integrator_chain_transceivers(
    ctx: Context<InitIntegratorChainTransceivers>,
    chain_id: u16,
) -> Result<()> {
    ctx.accounts
        .integrator_chain_transceivers
        .set_inner(IntegratorChainTransceivers {
            bump: ctx.bumps.integrator_chain_transceivers,
            chain_id,
            owner: ctx.accounts.owner.key(),
            next_in_transceiver_id: 0,
            next_out_transceiver_id: 0,
            in_transceiver_bitmap: Bitmap::new(),
            out_transceiver_bitmap: Bitmap::new(),
        });

    Ok(())
}