use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::EndpointError,
    event::MessageSent,
    state::{IntegratorChainConfig, OutboxMessage, SequenceTracker},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub integrator_program_id: Pubkey,
    pub integrator_program_pda_bump: u8,
    pub dst_chain: u16,
    pub dst_addr: [u8; 32],
    pub payload_hash: [u8; 32],
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: SendMessageArgs)]
pub struct SendMessage<'info> {
    // Payer pays for the init of `outbox_message`
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The PDA of the integrator program.
    /// This makes sure that only the integrator program is authorized to use this ix
    #[account(
        seeds = [b"endpoint_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program_pda: Signer<'info>,

    #[account(
        seeds = [
            IntegratorChainConfig::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.dst_chain.to_be_bytes().as_ref()
        ],
        bump = integrator_chain_config.bump,
    )]
    pub integrator_chain_config: Account<'info, IntegratorChainConfig>,

    #[account(
        mut,
        seeds = [
            SequenceTracker::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
        ],
        bump = sequence_tracker.bump,
    )]
    pub sequence_tracker: Account<'info, SequenceTracker>,

    #[account(
        init,
        payer = payer,
        space = 8 + OutboxMessage::INIT_SPACE,
    )]
    pub outbox_message: Account<'info, OutboxMessage>,

    pub system_program: Program<'info, System>,
}

/// Creates a new outbox message and initializes it with the provided information.
///
/// This function performs the following steps:
/// 1. Checks if there are any enabled send adapters for the destination chain.
/// 2. Initializes a new `OutboxMessage` account with the provided information.
/// 3. Increments the sequence number in the `SequenceTracker` account.
/// 4. Emits a `MessageSent` event with details about the sent message.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved.
/// * `args` - The arguments for the instruction, including:
///   * `integrator_program_id`: The program ID of the integrator.
///   * `integrator_program_pda_bump`: The bump seed for the integrator program PDA.
///   * `dst_chain`: The destination chain ID.
///   * `dst_addr`: The destination address as a UniversalAddress.
///   * `payload_hash`: The hash of the message payload.
///
/// # Errors
///
/// This function will return an error if:
/// * There are no enabled send adapters for the destination chain (EndpointError::AdapterNotEnabled).
///
/// # Side Effects
///
/// * Initializes a new `OutboxMessage` account.
/// * Increments the sequence number in the `SequenceTracker` account.
///
/// # Events
///
/// Emits a `MessageSent` event
pub fn send_message(ctx: Context<SendMessage>, args: SendMessageArgs) -> Result<()> {
    // Check if there are any enabled send adapters for destination chain
    require!(
        !ctx.accounts
            .integrator_chain_config
            .send_adapter_bitmap
            .is_empty(),
        EndpointError::AdapterNotEnabled
    );

    // Create and initialize the outbox message
    ctx.accounts.outbox_message.set_inner(OutboxMessage {
        src_addr: args.integrator_program_id.to_bytes(),
        sequence: ctx.accounts.sequence_tracker.next_sequence(),
        dst_chain: args.dst_chain,
        dst_addr: args.dst_addr,
        payload_hash: args.payload_hash,
        outstanding_adapters: ctx.accounts.integrator_chain_config.send_adapter_bitmap,
        refund_recipient: ctx.accounts.payer.key(),
    });

    emit_cpi!(MessageSent {
        sender: UniversalAddress::from(args.integrator_program_id),
        sequence: ctx.accounts.sequence_tracker.sequence,
        recipient: UniversalAddress::from_bytes(args.dst_addr),
        recipient_chain: args.dst_chain,
        payload_digest: args.payload_hash,
    });

    Ok(())
}
