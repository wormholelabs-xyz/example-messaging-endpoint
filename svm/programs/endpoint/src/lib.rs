use anchor_lang::prelude::*;

pub mod error;
pub mod event;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("FMPF1RnXz1vvZ6eovoEQqMPXYRUgYqFKFMXzTJkbWWVD");

/// The main program module for the Endpoint
#[program]
pub mod endpoint {
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

    /// Registers a new adapter for an integrator
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `EnableAdapterArgs` struct containing:
    ///     * `integrator_program` - The program id of the integrator_program
    ///     * `adapter_program_id` - The address of the adapter to register
    pub fn add_adapter(ctx: Context<AddAdapter>, args: AddAdapterArgs) -> Result<()> {
        instructions::add_adapter::add_adapter(ctx, args)
    }

    /// Sets an adapter as a receive adapter for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `EnableAdapterArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the adapter is being set
    ///     * `adapter` - The Pubkey of the adapter to be set
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn enable_recv_adapter(ctx: Context<EnableAdapter>, args: AdapterInfoArgs) -> Result<()> {
        instructions::enable_adapter::enable_recv_adapter(ctx, args)
    }

    /// Sets an adapter as a send adapter for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `EnableAdapterArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the adapter is being set
    ///     * `adapter` - The Pubkey of the adapter to be set
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn enable_send_adapter(ctx: Context<EnableAdapter>, args: AdapterInfoArgs) -> Result<()> {
        instructions::enable_adapter::enable_send_adapter(ctx, args)
    }

    /// Disables a receive adapter for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `DisableAdapterArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the adapter is being disabled
    ///     * `adapter` - The Pubkey of the adapter to be disabled
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn disable_recv_adapter(ctx: Context<DisableAdapter>, args: AdapterInfoArgs) -> Result<()> {
        instructions::disable_adapter::disable_recv_adapter(ctx, args)
    }

    /// Disables a send adapter for a specific chain
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `DisableAdapterArgs` struct containing:
    ///     * `chain_id` - The ID of the chain for which the adapter is being disabled
    ///     * `adapter` - The Pubkey of the adapter to be disabled
    ///     * `integrator_program` - The Pubkey of the integrator program
    pub fn disable_send_adapter(ctx: Context<DisableAdapter>, args: AdapterInfoArgs) -> Result<()> {
        instructions::disable_adapter::disable_send_adapter(ctx, args)
    }

    /// Updates the admin of an IntegratorConfig account
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction, containing:
    ///     * `admin` - The current admin (signer)
    ///     * `integrator_config` - The IntegratorConfig account to update
    /// * `args` - The `UpdateAdminArgs` struct containing:
    ///     * `new_admin` - The public key of the new admin
    ///     * `integrator_program_id` - The program ID of the integrator
    pub fn update_admin(ctx: Context<UpdateAdmin>, args: UpdateAdminArgs) -> Result<()> {
        instructions::update_admin::update_admin(ctx, args)
    }

    /// Initiates the transfer of admin rights for an IntegratorConfig account
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `TransferAdminArgs` struct containing:
    ///     * `new_admin` - The public key of the new admin
    ///     * `integrator_program_id` - The program ID of the integrator
    pub fn transfer_admin(ctx: Context<TransferAdmin>, args: TransferAdminArgs) -> Result<()> {
        instructions::transfer_admin::transfer_admin(ctx, args)
    }

    /// Claims the admin rights for an IntegratorConfig account
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn claim_admin(ctx: Context<ClaimAdmin>) -> Result<()> {
        instructions::transfer_admin::claim_admin(ctx)
    }

    /// Discards the admin role for an IntegratorConfig account, making it immutable
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    pub fn discard_admin(ctx: Context<DiscardAdmin>) -> Result<()> {
        instructions::discard_admin::discard_admin(ctx)
    }

    /// Sends a message through the endpoint
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `SendMessageArgs` struct containing:
    ///     * `integrator_program_id` - The program ID of the integrator
    ///     * `integrator_program_pda_bump` - The bump for the integrator_program_pda derivation
    ///     * `dst_chain` - The destination chain ID
    ///     * `dst_addr` - The destination address
    ///     * `payload_hash` - The hash of the message payload
    pub fn send_message(ctx: Context<SendMessage>, args: SendMessageArgs) -> Result<()> {
        instructions::send_message::send_message(ctx, args)
    }

    /// Picks up a message from the outbox
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction containing:
    ///     * `outbox_message` - The outbox message to pick up
    ///     * `adapter_info` - The adapter info account
    ///     * `adapter_pda` - The adapter PDA signer
    /// * `args` - The `PickUpMessageArgs` struct containing:
    ///     * `adapter_program_id` - The program ID of the adapter
    ///     * `adapter_pda_bump` - The bump for the adapter PDA
    pub fn pick_up_message(ctx: Context<PickUpMessage>, args: PickUpMessageArgs) -> Result<()> {
        instructions::pick_up_message::pick_up_message(ctx, args)
    }

    /// Attests to a message
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `AttestMessageArgs` struct containing:
    ///     * `adapter_program_id` - The program ID of the adapter
    ///     * `adapter_pda_bump` - The bump for the adapter PDA
    ///     * `src_chain` - The source chain ID
    ///     * `src_addr` - The source address
    ///     * `sequence` - The sequence number
    ///     * `dst_chain` - The destination chain ID
    ///     * `integrator_program_id` - The program ID of the integrator, aka dst_addr
    ///     * `payload_hash` - The hash of the message payload
    pub fn attest_message(ctx: Context<AttestMessage>, args: AttestMessageArgs) -> Result<()> {
        instructions::attest_message::attest_message(ctx, args)
    }

    /// Executes a message
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction
    /// * `args` - The `ExecMessageArgs` struct containing:
    ///     * `integrator_program_pda_bump` - The bump for the integrator program PDA
    ///     * `src_chain` - The source chain ID
    ///     * `src_addr` - The source address
    ///     * `sequence` - The sequence number
    ///     * `dst_chain` - The destination chain ID
    ///     * `integrator_program_id` - The program ID of the integrator, aka dst_addr
    ///     * `payload_hash` - The hash of the message payload
    pub fn exec_message(
        ctx: Context<ExecMessage>,
        args: exec_message::ExecMessageArgs,
    ) -> Result<()> {
        exec_message::exec_message(ctx, args)
    }

    /// Receives a message that has been attested to.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the instruction, containing the accounts involved
    /// * `args` - The `RecvMessageArgs` struct containing:
    ///     * `integrator_program_pda_bump` - The bump seed for the integrator program PDA
    ///     * `src_chain` - The source chain ID
    ///     * `src_addr` - The source address as a UniversalAddress
    ///     * `sequence` - The sequence number of the message
    ///     * `dst_chain` - The destination chain ID
    ///     * `integrator_program_id` - The program ID of the integrator, aka dst_addr
    ///     * `payload_hash` - The hash of the message payload
    pub fn recv_message(
        ctx: Context<RecvMessage>,
        args: recv_message::RecvMessageArgs,
    ) -> Result<()> {
        recv_message::recv_message(ctx, args)
    }
}
