use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("7qtLhNMdb9dNAWwFvNBMok64EJrS1toY9TQoedVhU1xp");

/// The main program module for the GMP Router
#[program]
pub mod router {
    use super::*;

    /// Initializes the GMP Router
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    /// Initializes the integrator config
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn init_integrator_config(ctx: Context<InitIntegratorConfig>) -> Result<()> {
        // TODO: fix spelling
        instructions::initialize_integrator_config::init_integrator_config(ctx)
    }

    /// Registers a new transceiver for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `chain_id` - The ID of the chain for which the transceiver is being registered
    /// * `transceiver_type` - The type of the transceiver (In or Out)
    /// * `transceiver_address` - The public key of the transceiver address
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

    /// Initializes the chain transceivers for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `chain_id` - The ID of the chain for which the transceivers are being initialized
    pub fn init_integrator_chain_transceivers(
        ctx: Context<InitIntegratorChainTransceivers>,
        chain_id: u16,
    ) -> Result<()> {
        instructions::init_integrator_chain_transceivers::init_integrator_chain_transceivers(
            ctx, chain_id,
        )
    }

    /// Transfers the ownership of integrator chain transceivers
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `new_owner` - The public key of the new owner
    pub fn transfer_integrator_chain_transceivers_ownership(
        ctx: Context<TransferIntegratorChainTransceiversOwnership>,
        new_owner: Pubkey,
    ) -> Result<()> {
        // Calls the function to transfer ownership of the integrator chain transceivers
        transfer_integrator_chain_transceivers_ownership::transfer_integrator_chain_transceivers_ownership(ctx, new_owner)
    }
}
