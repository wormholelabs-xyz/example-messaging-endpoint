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
    /// * `args` - The arguments for registration, including the integrator program ID
    pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
        instructions::register::register(ctx, args)
    }

    /// Registers a new transceiver for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The arguments for registering a transceiver, including the integrator program ID and transceiver address
    pub fn register_transceiver(
        ctx: Context<RegisterTransceiver>,
        args: RegisterTransceiverArgs,
    ) -> Result<()> {
        instructions::register_transceiver::register_transceiver(ctx, args)
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

    /// Transfers adminship of the IntegratorConfig to a new admin
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> {
        instructions::update_admin::update_admin(ctx)
    }
}
