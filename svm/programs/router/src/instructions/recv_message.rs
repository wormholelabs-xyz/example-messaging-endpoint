use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::RouterError,
    state::{AttestationInfo, IntegratorChainConfig},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RecvMessageArgs {
    pub integrator_program_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: RecvMessageArgs)]
pub struct RecvMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"router_integrator"],
        bump = args.integrator_program_pda_bump,
        seeds::program = args.dst_addr.to_pubkey()
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
                args.dst_addr,
                args.payload_hash
            )
        ],
        bump = attestation_info.bump,
    )]
    pub attestation_info: Account<'info, AttestationInfo>,

    pub system_program: Program<'info, System>,
}

// We won't double check for `no attestations here`. If it reaches here, it means the `AttestationInfo`
// is already initialized. In other words, either `attest_message` or `exec_message` has been invoked.
// In case of `exec_message`, `AlreadyExecuted` will be thrown
pub fn recv_message(ctx: Context<RecvMessage>, _args: RecvMessageArgs) -> Result<(u128, u128)> {
    let attestation_info = &mut ctx.accounts.attestation_info;
    let integrator_chain_config = &ctx.accounts.integrator_chain_config;

    // Check if the message has already been executed
    require!(!attestation_info.executed, RouterError::AlreadyExecuted);

    // Mark the message as executed
    attestation_info.executed = true;

    // Return the enabled receive Transceivers for that chain along with the attestations
    Ok((
        integrator_chain_config.recv_transceiver_bitmap.as_value(),
        attestation_info.attested_transceivers.as_value(),
    ))
}
