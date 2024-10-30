use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{AttestationInfo, AttestationInfoArgs},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecMessageArgs {
    pub integrator_program_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
    pub message_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: ExecMessageArgs)]
pub struct ExecMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The PDA of the integrator program.
    /// This makes sure that the one calling this is the integrator program
    #[account(
        seeds = [b"router_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.integrator_program_id,
    )]
    pub integrator_program_pda: Signer<'info>,

    /// The attestation info account
    /// This account is initialized if it doesn't exist
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + AttestationInfo::INIT_SPACE,
        seeds = [
            AttestationInfo::SEED_PREFIX,
            &args.message_hash
        ],
        bump
    )]
    pub attestation_info: Account<'info, AttestationInfo>,

    pub system_program: Program<'info, System>,
}

/// Executes a message in the router program
///
/// This function is responsible for marking a message as executed. It checks if the message
/// has already been executed, initializes the attestation info if it's newly created,
/// and then marks the message as executed.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `args` - The arguments for the exec_message instruction:
///   - `integrator_program_pda_bump`: The bump seed for the integrator program's PDA
///   - `src_chain`: The source chain ID
///   - `src_addr`: The source address (UniversalAddress)
///   - `sequence`: The sequence number of the message
///   - `dst_chain`: The destination chain ID
///   - `dst_addr`: The destination address (UniversalAddress)
///   - `payload_hash`: The hash of the message payload
///
/// # Returns
///
/// Returns `Ok(())` if the message is successfully executed, or an error if it fails
pub fn exec_message(ctx: Context<ExecMessage>, args: ExecMessageArgs) -> Result<()> {
    let attestation_info = &mut ctx.accounts.attestation_info;

    // Check if the message has already been executed
    require!(!attestation_info.executed, RouterError::AlreadyExecuted);

    // If the attestation_info is newly created, initialize it
    if attestation_info.src_chain == 0 {
        let args = AttestationInfoArgs {
            bump: ctx.bumps.attestation_info,
            src_chain: args.src_chain,
            src_addr: args.src_addr,
            sequence: args.sequence,
            dst_chain: args.dst_chain,
            dst_addr: UniversalAddress::from_pubkey(&args.integrator_program_id),
            payload_hash: args.payload_hash,
            message_hash: args.message_hash,
        };

        attestation_info.set_inner(AttestationInfo::new(args)?);
    }

    // Mark the message as executed
    attestation_info.executed = true;

    Ok(())
}
