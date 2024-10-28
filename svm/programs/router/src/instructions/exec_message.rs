use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{AttestationInfo, IntegratorChainConfig, TransceiverInfo},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecMessageArgs {
    pub integrator_program_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
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
        seeds::program = args.dst_addr.to_pubkey(),
    )]
    pub integrator_program_pda: Signer<'info>,

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

pub fn exec_message(ctx: Context<ExecMessage>, args: ExecMessageArgs) -> Result<()> {
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
