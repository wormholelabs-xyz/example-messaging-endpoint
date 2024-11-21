use crate::{
    error::EndpointError,
    event::MessagePickedUp,
    state::{AdapterInfo, OutboxMessage},
};
use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PickUpMessageArgs {
    pub adapter_program_id: Pubkey,
    pub adapter_pda_bump: u8,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: PickUpMessageArgs)]
pub struct PickUpMessage<'info> {
    /// The outbox message account to be picked up
    /// This account is mutable so we can update the `outstanding_adapters` state
    #[account(
        mut,
        has_one = refund_recipient
    )]
    pub outbox_message: Account<'info, OutboxMessage>,

    /// The adapter info account
    /// This account contains index of the adapter picking up the message
    #[account(
        seeds = [
            AdapterInfo::SEED_PREFIX,
            outbox_message.src_addr.as_ref(),
            args.adapter_program_id.as_ref(),
        ],
        bump = adapter_info.bump,
    )]
    pub adapter_info: Account<'info, AdapterInfo>,

    /// The adapter PDA account, used for signing
    /// This ensures that only the authorized adapter can pick up the message
    #[account(
        seeds = ["adapter_pda".as_bytes()],
        bump = args.adapter_pda_bump,
        seeds::program = args.adapter_program_id
    )]
    pub adapter_pda: Signer<'info>,

    /// The account that will receive the rent from closing the outbox message account
    #[account(mut)]
    /// CHECK: This is an account for receiving the rent refund
    pub refund_recipient: AccountInfo<'info>,

    /// The system program account
    pub system_program: Program<'info, System>,
}

/// Instruction for picking up a message from the outbox.
///
/// This function performs the following steps:
/// 1. Checks if the message is available for pick up by this adapter.
/// 2. Marks the message as picked up by updating the `outstanding_adapters` bitmap.
/// 3. Emits a MessagePickedUp event.
/// 4. Closes the outbox message account if all adapters have picked up the message.
///
/// # Arguments
///
/// * `args` - The arguments for the instruction, including:
///   * `adapter_program_id`: The Pubkey of the adapter program.
///   * `adapter_pda_bump`: The bump seed for the adapter's PDA.
///
/// # Returns
///
/// Returns `Ok(())` if the message is successfully picked up, or an error otherwise.
///
/// * `ctx` - The context of the instruction, containing the accounts involved.
///
/// # Errors
///
/// This function will return an error if:
/// * The message has already been picked up by this adapter (EndpointError::MessageAlreadyPickedUp).
/// * There's an issue updating the `outstanding_adapters` bitmap.
/// * There's an issue closing the outbox message account when all adapters have picked up the message.
///
/// # Events
///
/// Emits a `MessagePickedUp` event
pub fn pick_up_message(ctx: Context<PickUpMessage>, args: PickUpMessageArgs) -> Result<()> {
    let outbox_message = &mut ctx.accounts.outbox_message;
    let adapter_info = &ctx.accounts.adapter_info;

    // Get the index of the adapter
    let adapter_index = adapter_info.index;

    // Check if the message is available for pick up by this adapter
    require!(
        outbox_message
            .outstanding_adapters
            .get(adapter_index)
            .unwrap_or(false),
        EndpointError::MessageAlreadyPickedUp
    );

    // Mark the message as picked up by this adapter
    outbox_message
        .outstanding_adapters
        .set(adapter_index, false)?;

    emit_cpi!(MessagePickedUp {
        src_addr: UniversalAddress::from_bytes(outbox_message.src_addr),
        sequence: outbox_message.sequence,
        dst_chain: outbox_message.dst_chain,
        dst_addr: UniversalAddress::from_bytes(outbox_message.dst_addr),
        payload_hash: outbox_message.payload_hash,
        adapter: args.adapter_program_id,
        remaining_adapters: outbox_message.outstanding_adapters.as_value(),
    });

    // Close `outbox_message` account if all adapters have picked up the message
    if outbox_message.outstanding_adapters.as_value() == 0 {
        ctx.accounts
            .outbox_message
            .close(ctx.accounts.refund_recipient.to_account_info())?
    }

    Ok(())
}
