use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{IntegratorChainConfig, OutboxMessage, OutboxMessageKey},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub integrator_program_id: Pubkey,
    pub integrator_program_pda_bump: u8,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: SendMessageArgs)]
pub struct SendMessage<'info> {
    // Payer pays for the init of `outbox_message`
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"router_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program_pda: Signer<'info>,

    #[account(
        mut,
        seeds = [
            OutboxMessageKey::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
        ],
        bump = outbox_message_key.bump,
    )]
    pub outbox_message_key: Account<'info, OutboxMessageKey>,

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
        init,
        payer = payer,
        space = 8 + OutboxMessage::INIT_SPACE,
    )]
    pub outbox_message: Account<'info, OutboxMessage>,

    pub system_program: Program<'info, System>,
}

pub fn send_message(ctx: Context<SendMessage>, args: SendMessageArgs) -> Result<()> {
    // Check if there are any enabled send transceivers for destination chain
    require!(
        !ctx.accounts
            .integrator_chain_config
            .send_transceiver_bitmap
            .is_empty(),
        RouterError::TransceiverNotEnabled
    );

    // Create and initialize the outbox message
    ctx.accounts.outbox_message.set_inner(OutboxMessage {
        src_addr: UniversalAddress::from(ctx.accounts.integrator_program_pda.key()),
        sequence: ctx.accounts.outbox_message_key.next_sequence(),
        dst_chain: args.dst_chain,
        dst_addr: args.dst_addr,
        payload_hash: args.payload_hash,
        outstanding_transceivers: ctx
            .accounts
            .integrator_chain_config
            .send_transceiver_bitmap
            .clone(),
    });

    Ok(())
}
