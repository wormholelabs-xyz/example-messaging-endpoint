use crate::error::RouterError;
use crate::state::{Config, Integrator, IntegratorChainTransceivers, RegisteredTransceiver};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(chain_id: u16)]
pub struct RegisterTransceiver<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
        constraint = !config.paused @ RouterError::ProgramPaused,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [Integrator::SEED_PREFIX, integrator.id.to_le_bytes().as_ref()],
        bump = integrator.bump,
        has_one = authority @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator: Account<'info, Integrator>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,

    #[account(
        init,
        payer = payer,
        space = 8 + RegisteredTransceiver::INIT_SPACE,
        seeds = [
            RegisteredTransceiver::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
            integrator_chain_transceivers.next_transceiver_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    chain_id: u16,
    transceiver_address: Pubkey,
) -> Result<()> {
    let chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;
    let transceiver_id = chain_transceivers.next_transceiver_id;

    // Ensure we don't exceed the maximum number of transceivers
    if transceiver_id >= 256 {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Update the bitmap
    let bit_position = transceiver_id % 64;
    chain_transceivers.transceiver_bitmap |= 1u64 << bit_position;

    // Increment the next_transceiver_id
    chain_transceivers.next_transceiver_id += 1;

    // Initialize the RegisteredTransceiver account
    let registered_transceiver = &mut ctx.accounts.registered_transceiver;
    registered_transceiver.integrator_id = ctx.accounts.integrator.id;
    registered_transceiver.id = transceiver_id;
    registered_transceiver.chain_id = chain_id;
    registered_transceiver.address = transceiver_address;

    Ok(())
}
