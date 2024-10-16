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

    /// Registers an integrator and initializes their configuration
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `RegisterArgs` struct containing:
    ///     * `integrator_program_id` - The program ID of the integrator
    ///     * `integrator_program_pda_bump` - The bump for the integrator_program_pda derivation
    pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
        instructions::register::register(ctx, args)
    }

    /// Registers a new transceiver for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `RegisterTransceiverArgs` struct containing:
    ///     * `integrator_program` - The program id of the integrator_program
    ///     * `transceiver_address` - The address of the transceiver to register
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
    /// * `args` - The `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being set
    ///     * `transceiver` - The Pubkey of the transceiver to be set
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn set_recv_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::set_recv_transceiver(ctx, args)
    }

    /// Sets a transceiver as a send transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `SetTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being set
    ///     * `transceiver` - The Pubkey of the transceiver to be set
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn set_send_transceiver(
        ctx: Context<SetTransceiver>,
        args: SetTransceiverArgs,
    ) -> Result<()> {
        instructions::set_transceivers::set_send_transceiver(ctx, args)
    }

    /// Disables a receive transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `DisableTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being disabled
    ///     * `transceiver` - The Pubkey of the transceiver to be disabled
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn disable_recv_transceiver(
        ctx: Context<DisableTransceiver>,
        args: DisableTransceiverArgs,
    ) -> Result<()> {
        instructions::disable_transceivers::disable_recv_transceiver(ctx, args)
    }

    /// Disables a send transceiver for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `DisableTransceiverArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the transceiver is being disabled
    ///     * `transceiver` - The Pubkey of the transceiver to be disabled
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn disable_send_transceiver(
        ctx: Context<DisableTransceiver>,
        args: DisableTransceiverArgs,
    ) -> Result<()> {
        instructions::disable_transceivers::disable_send_transceiver(ctx, args)
    }

    /// Transfers adminship of the IntegratorConfig to a new admin
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction, containing:
    ///     * `authority` - The current admin (signer)
    ///     * `new_admin` - The account of the new admin
    ///     * `integrator_config` - The IntegratorConfig account to update
    pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> {
        instructions::update_admin::update_admin(ctx)
    }
}
