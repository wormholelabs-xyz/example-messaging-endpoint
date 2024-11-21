use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::EndpointError,
    event::MessageReceived,
    state::{AttestationInfo, IntegratorChainConfig},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RecvMessageArgs {
    pub integrator_program_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: RecvMessageArgs)]
pub struct RecvMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The PDA of the integrator program.
    /// This makes sure that the one calling this is the integrator program
    #[account(
        seeds = [b"endpoint_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id
    )]
    pub integrator_program_pda: Signer<'info>,

    /// The integrator chain config account
    /// This is required to read the enabled_bitmap from
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
    /// This throws when there is no attestation as there is no account initialized yet
    #[account(
        mut,
        seeds = [
            AttestationInfo::SEED_PREFIX,
            &AttestationInfo::compute_message_hash(
                args.src_chain,
                args.src_addr,
                args.sequence,
                args.dst_chain,
                args.integrator_program_id.to_bytes(),
                args.payload_hash
            )
        ],
        bump = attestation_info.bump,
    )]
    pub attestation_info: Account<'info, AttestationInfo>,

    pub system_program: Program<'info, System>,
}

/// Receives a message that has been attested to in the endpoint program
///
/// This function performs the following steps:
/// 1. Checks if the message has already been executed.
/// 2. Marks the message as executed.
/// 3. Emits a MessageReceived event.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `args` - The arguments for the recv_message instruction, including:
///   * `integrator_program_pda_bump`: The bump seed for the integrator program PDA.
///   * `src_chain`: The source chain ID.
///   * `src_addr`: The source address as a UniversalAddress.
///   * `sequence`: The sequence number of the message.
///   * `dst_chain`: The destination chain ID.
///   * `integrator_program_id`: The public key of the integrator program.
///   * `payload_hash`: The hash of the message payload.
///
/// # Returns
///
/// Returns `Ok(())` if the message is successfully received, or an error if it fails
///
/// # Errors
///
/// This function will return an error if:
/// * The message has already been executed (EndpointError::AlreadyExecuted)
///
/// # Events
///
/// Emits a `MessageReceived` event
///
/// # Notes
///
/// We don't double-check for `no attestations here`. If it reaches this point,
/// it means the `AttestationInfo` is already initialized. In other words,
/// either `attest_message` or `exec_message` has been invoked previously.
/// In the case of `exec_message`, `AlreadyExecuted` will be thrown.
pub fn recv_message(ctx: Context<RecvMessage>, _args: RecvMessageArgs) -> Result<()> {
    let attestation_info = &mut ctx.accounts.attestation_info;

    // Check if the message has already been executed
    require!(!attestation_info.executed, EndpointError::AlreadyExecuted);

    // There is no need to check for the src_chain and dst_chain validity since they
    // are check during the init of attestation_info in either `exec_message` or `attest_message`

    // Mark the message as executed
    attestation_info.executed = true;

    emit_cpi!(MessageReceived {
        message_hash: attestation_info.message_hash,
        src_chain: attestation_info.src_chain,
        src_addr: UniversalAddress::from_bytes(attestation_info.src_addr),
        sequence: attestation_info.sequence,
        dst_chain: attestation_info.dst_chain,
        dst_addr: UniversalAddress::from_bytes(attestation_info.dst_addr),
        payload_hash: attestation_info.payload_hash,
        enabled_bitmap: ctx
            .accounts
            .integrator_chain_config
            .recv_adapter_bitmap
            .as_value(),
        attested_bitmap: attestation_info.attested_adapters.as_value(),
    });

    // Return the enabled receive Adapters for that chain along with the attestations
    Ok(())
}
