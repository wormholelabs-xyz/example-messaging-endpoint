use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{error::EndpointError, state::AttestationInfo};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RecvMessageArgs {
    pub integrator_program_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
}

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
                UniversalAddress::from_pubkey(&args.integrator_program_id),
                args.payload_hash
            )
        ],
        bump = attestation_info.bump,
    )]
    pub attestation_info: Account<'info, AttestationInfo>,

    pub system_program: Program<'info, System>,
}

/// This instruction is called to receive a message that has been attested to.
/// It marks the message as executed and returns the enabled receive adapters
/// for the source chain along with the attestations.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts involved.
/// * `args` - The arguments for the instruction, including:
///   * `integrator_program_pda_bump`: The bump seed for the integrator program PDA.
///   * `src_chain`: The source chain ID.
///   * `src_addr`: The source address as a UniversalAddress.
///   * `sequence`: The sequence number of the message.
///   * `dst_chain`: The destination chain ID.
///   * `dst_addr`: The destination address as a UniversalAddress.
///   * `payload_hash`: The hash of the message payload.
///
/// # Returns
///
/// A tuple containing:
/// * The bitmap of enabled receive adapters for the source chain.
/// * The bitmap of adapters that have attested to the message.
///
/// # Errors
///
/// This function will return an error if:
/// * The message has already been executed (EndpointError::AlreadyExecuted).
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

    // Return the enabled receive Adapters for that chain along with the attestations
    Ok(())
}
