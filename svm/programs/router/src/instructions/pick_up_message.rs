use crate::{
    error::RouterError,
    state::{OutboxMessage, TransceiverInfo},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct PickUpMessage<'info> {
    /// The outbox message account to be picked up
    /// This account is mutable so we can update the `outstanding_transceivers` state
    #[account(mut)]
    pub outbox_message: Account<'info, OutboxMessage>,

    /// The transceiver info account
    /// This account contains index of the transceiver picking up the message
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// The transceiver PDA account, used for signing
    /// This ensures that only the authorized transceiver can pick up the message
    #[account(
        seeds = ["transceiver_pda".as_bytes()],
        bump,
        seeds::program = transceiver_info.transceiver_program_id
    )]
    pub transceiver_pda: Signer<'info>,

    /// The account that will receive the rent from closing the outbox message account
    #[account(mut)]
    /// CHECK: This is an account for receiving the rent refund
    pub refund_recipient: AccountInfo<'info>,

    /// The system program account
    pub system_program: Program<'info, System>,
}

/// Instruction for picking up a message from the outbox.
///
/// This instruction allows a transceiver to pick up a message from the outbox.
/// It updates the `outstanding_transceivers` bitmap to mark the message as picked up
/// by the current transceiver. If all transceivers have picked up the message,
/// the outbox message account is closed and the rent is refunded.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved.
///
/// # Errors
///
/// This function will return an error if:
/// * The message has already been picked up by this transceiver.
/// * There's an issue updating the `outstanding_transceivers` bitmap.
/// * There's an issue closing the outbox message account when all transceivers have picked up the message.
pub fn pick_up_message(ctx: Context<PickUpMessage>) -> Result<()> {
    let outbox_message = &mut ctx.accounts.outbox_message;
    let transceiver_info = &ctx.accounts.transceiver_info;

    // Get the index of the transceiver
    let transceiver_index = transceiver_info.index;

    msg!("Transceiver Info: {:?}", transceiver_info);
    msg!(
        "Outstanding Transceivers: {:?}",
        outbox_message.outstanding_transceivers
    );
    // Check if the message is available for pick up by this transceiver
    require!(
        outbox_message
            .outstanding_transceivers
            .get(transceiver_index)
            .unwrap_or(false),
        RouterError::MessageAlreadyPickedUp
    );

    // Mark the message as picked up by this transceiver
    outbox_message
        .outstanding_transceivers
        .set(transceiver_index, false)
        .map_err(|_| RouterError::MessageAlreadyPickedUp)?;

    // Check if all transceivers have picked up the message
    if outbox_message.outstanding_transceivers.as_value() == 0 {
        ctx.accounts
            .outbox_message
            .close(ctx.accounts.refund_recipient.to_account_info())?
    }

    Ok(())
}
