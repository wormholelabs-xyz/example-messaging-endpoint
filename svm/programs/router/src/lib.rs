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

    /// Initializes the integrator config
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn init_integrator_config(ctx: Context<Register>) -> Result<()> {
        instructions::register::register(ctx)
    }

    /// Initializes the chain transceivers for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `chain_id` - The ID of the chain for which the transceivers are being initialized
    pub fn initialize_integrator_chain_transceivers(
        ctx: Context<InitializeIntegratorChainTransceivers>,
        chain_id: u16,
    ) -> Result<()> {
        instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers(
            ctx, chain_id,
        )
    }

    /// Registers a new transceiver for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `transceiver_address` - The address of the transceiver to be registered
    pub fn register_transceiver(ctx: Context<RegisterTransceiver>) -> Result<()> {
        instructions::register_transceiver::register_transceiver(ctx)
    }

    /// Sets a transceiver as a receive transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - A `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being set
    pub fn set_recv_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::set_recv_transceiver(ctx, args)
    }

    /// Disables a receive transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - A `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being disabled
    pub fn disable_recv_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::disable_recv_transceiver(ctx, args)
    }

    /// Sets a transceiver as a send transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - A `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being set
    pub fn set_send_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::set_send_transceiver(ctx, args)
    }

    /// Disables a send transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - A `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being disabled
    pub fn disable_send_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::disable_send_transceiver(ctx, args)
    }

    /// Transfers ownership of the IntegratorConfig to a new owner
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn transfer_integrator_config_ownership(ctx: Context<UpdateAdmin>) -> Result<()> {
        instructions::update_admin::update_admin(ctx)
    }
}
