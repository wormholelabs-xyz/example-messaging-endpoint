use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{AttestationInfo, IntegratorChainConfig, TransceiverInfo},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AttestMessageArgs {
    pub transceiver_program_id: Pubkey,
    pub transceiver_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: AttestMessageArgs)]
pub struct AttestMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The transceiver info account
    #[account(
        seeds = [
            TransceiverInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.transceiver_program_id.as_ref(),
        ],
        bump = transceiver_info.bump,
    )]
    pub transceiver_info: Account<'info, TransceiverInfo>,

    /// The transceiver PDA signing account.
    /// This check makes sure that only the transceiver program is authorised to call this message
    #[account(
        seeds = ["transceiver_pda".as_bytes()],
        bump = args.transceiver_pda_bump,
        seeds::program = args.transceiver_program_id
    )]
    pub transceiver_pda: Signer<'info>,

    /// The integrator chain config account
    #[account(
        seeds = [
            IntegratorChainConfig::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
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
                UniversalAddress::from_pubkey(&args.integrator_program_id),
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
/// * `args` - The arguments for the instruction, containing message details:
///   - `transceiver_pda_bump`: The bump seed for the transceiver's PDA
///   - `src_chain`: The source chain ID
///   - `src_addr`: The source address (UniversalAddress)
///   - `sequence`: The sequence number of the message
///   - `dst_chain`: The destination chain ID
///   - `dst_addr`: The destination address (UniversalAddress)
///   - `payload_hash`: The hash of the message payload
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
    // It is fine to check for initialization using `src_chain == 0` as
    // `IntegratorChainConfig` and `AttestationInfo` can never have chain_id that is 0
    if attestation_info.src_chain == 0 {
        attestation_info.set_inner(AttestationInfo::new(
            ctx.bumps.attestation_info,
            args.src_chain,
            args.src_addr,
            args.sequence,
            args.dst_chain,
            UniversalAddress::from_pubkey(&args.integrator_program_id),
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
