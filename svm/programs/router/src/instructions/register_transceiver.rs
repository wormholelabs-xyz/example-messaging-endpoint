use crate::error::RouterError;
use crate::state::{Config, Integrator, IntegratorChainTransceivers, RegisteredTransceiver};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TransceiverType {
    In,
    Out,
}

#[derive(Accounts)]
#[instruction(chain_id: u16, transceiver_type: TransceiverType)]
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
            {
                let transceiver_id = match transceiver_type {
                    TransceiverType::In => integrator_chain_transceivers.next_in_transceiver_id,
                    TransceiverType::Out => integrator_chain_transceivers.next_out_transceiver_id,
                };
                transceiver_id.to_le_bytes().as_ref()
            }    ],
        bump
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    pub system_program: Program<'info, System>,
}

pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    chain_id: u16,
    transceiver_type: TransceiverType,
    transceiver_address: Pubkey,
) -> Result<()> {
    let chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;

    let transceiver_id = match transceiver_type {
        TransceiverType::In => chain_transceivers.next_in_transceiver_id,
        TransceiverType::Out => chain_transceivers.next_out_transceiver_id,
    };

    // Ensure we don't exceed the maximum number of transceivers
    if transceiver_id >= IntegratorChainTransceivers::MAX_TRANSCEIVERS as u64 {
        return Err(RouterError::MaxTransceiversReached.into());
    }

    // Update the bitmap
    match transceiver_type {
        TransceiverType::In => {
            chain_transceivers.set_in_transceiver(transceiver_id as u8, true)?;
            chain_transceivers.next_in_transceiver_id += 1;
        },
        TransceiverType::Out => {
            chain_transceivers.set_out_transceiver(transceiver_id as u8, true)?;
            chain_transceivers.next_out_transceiver_id += 1;
        },
    }

    // Initialize the RegisteredTransceiver account
    let registered_transceiver = &mut ctx.accounts.registered_transceiver;
    registered_transceiver.integrator_id = ctx.accounts.integrator.id;
    registered_transceiver.id = transceiver_id;
    registered_transceiver.chain_id = chain_id;
    registered_transceiver.address = transceiver_address;

    Ok(())
}
