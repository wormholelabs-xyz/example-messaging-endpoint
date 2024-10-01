use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("7qtLhNMdb9dNAWwFvNBMok64EJrS1toY9TQoedVhU1xp");

#[program]
pub mod router {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        instructions::initialize::initialize(ctx, args)
    }

    pub fn register_integrator(ctx: Context<RegisterIntegrator>, authority: Pubkey) -> Result<()> {
        instructions::register_integrator::register_integrator(ctx, authority)
    }

    pub fn register_transceiver(
        ctx: Context<RegisterTransceiver>,
        chain_id: u16,
        transceiver_type: TransceiverType,
        transceiver_address: Pubkey,
    ) -> Result<()> {
        instructions::register_transceiver::register_transceiver(
            ctx,
            chain_id,
            transceiver_type,
            transceiver_address,
        )
    }

    pub fn init_integrator_chain_transceivers(
        ctx: Context<InitIntegratorChainTransceivers>,
        chain_id: u16,
    ) -> Result<()> {
        instructions::init_integrator_chain_transceivers::init_integrator_chain_transceivers(
            ctx, chain_id,
        )
    }
}
