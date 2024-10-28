use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{AttestationInfo, IntegratorChainConfig, TransceiverInfo},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AttestMessageArgs {
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: AttestMessageArgs)]
pub struct AttestMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The transceiver info account
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// The transceiver PDA account, used for signing
    #[account(
        seeds = ["transceiver_pda".as_bytes()],
        bump,
        seeds::program = transceiver_info.transceiver_program_id
    )]
    pub transceiver_pda: Signer<'info>,

    /// The integrator chain config account
    #[account(
        seeds = [
            IntegratorChainConfig::SEED_PREFIX,
            args.dst_addr.to_bytes().as_ref(),
            args.src_chain.to_be_bytes().as_ref()
        ],
        bump = integrator_chain_config.bump,
    )]
    pub integrator_chain_config: Account<'info, IntegratorChainConfig>,

    /// The attestation info account
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + AttestationInfo::INIT_SPACE,
        seeds = [
            AttestationInfo::SEED_PREFIX,
            &AttestationInfo::compute_message_hash(
                args.src_chain,
                args.src_addr,
                args.sequence,
                args.dst_chain,
                args.dst_addr,
                args.payload_hash
            )
        ],
        bump
    )]
    pub attestation_info: Account<'info, AttestationInfo>,

    pub system_program: Program<'info, System>,
}

/// Instruction handler for attest_message
///
/// This function allows a transceiver to attest to a message. It performs the following steps:
/// 1. Checks if the transceiver is enabled for receiving messages from the source chain.
/// 2. Initializes the attestation info account if it's newly created.
/// 3. Checks if the transceiver has already attested to this message.
/// 4. Marks the transceiver as having attested to the message.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `args` - The arguments for the instruction, containing message details
///
/// # Errors
///
/// This function will return an error if:
/// * The transceiver is not enabled for receiving messages from the source chain
/// * The transceiver has already attested to this message
///
/// # Returns
///
/// Returns `Ok(())` if the attestation is successful
pub fn attest_message(ctx: Context<AttestMessage>, args: AttestMessageArgs) -> Result<()> {
    let transceiver_info = &ctx.accounts.transceiver_info;
    let integrator_chain_config = &ctx.accounts.integrator_chain_config;
    let attestation_info = &mut ctx.accounts.attestation_info;

    // Check if the Transceiver is an enabled receive Transceiver for the Integrator and source chain
    require!(
        integrator_chain_config
            .recv_transceiver_bitmap
            .get(transceiver_info.index)
            .unwrap_or(false),
        RouterError::TransceiverNotEnabled
    );

    // If the attestation_info is newly created, initialize it
    if attestation_info.message_hash == [0; 32] {
        attestation_info.set_inner(AttestationInfo::new(
            ctx.bumps.attestation_info,
            args.src_chain,
            args.src_addr,
            args.sequence,
            args.dst_chain,
            args.dst_addr,
            args.payload_hash,
        )?);
    }

    // Check if the Transceiver has already attested
    require!(
        !attestation_info
            .attested_transceivers
            .get(transceiver_info.index)
            .unwrap_or(false),
        RouterError::DuplicateMessageAttestation
    );

    // Mark the Transceiver as having attested to the message
    attestation_info
        .attested_transceivers
        .set(transceiver_info.index, true)?;

    Ok(())
}

pub fn exec_message(ctx: Context<AttestMessage>, args: AttestMessageArgs) -> Result<()> {
    let attestation_info = &mut ctx.accounts.attestation_info;

    // Check if the message has already been executed
    require!(!attestation_info.executed, RouterError::AlreadyExecuted);

    // If the attestation_info is newly created, initialize it
    if attestation_info.message_hash == [0; 32] {
        attestation_info.set_inner(AttestationInfo::new(
            ctx.bumps.attestation_info,
            args.src_chain,
            args.src_addr,
            args.sequence,
            args.dst_chain,
            args.dst_addr,
            args.payload_hash,
        )?);
    }

    // Mark the message as executed
    attestation_info.executed = true;

    Ok(())
}

