use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::{
    error::EndpointError,
    event::MessageAttestedTo,
    state::{AdapterInfo, AttestationInfo, IntegratorChainConfig},
    CHAIN_ID,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AttestMessageArgs {
    pub adapter_program_id: Pubkey,
    pub adapter_pda_bump: u8,
    pub src_chain: u16,
    pub src_addr: [u8; 32],
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: AttestMessageArgs)]
pub struct AttestMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The adapter info account
    #[account(
        seeds = [
            AdapterInfo::SEED_PREFIX,
            args.integrator_program_id.as_ref(),
            args.adapter_program_id.as_ref(),
        ],
        bump = adapter_info.bump,
    )]
    pub adapter_info: Account<'info, AdapterInfo>,

    /// The adapter PDA signing account.
    /// This check makes sure that only the adapter program is authorised to call this message
    #[account(
        seeds = ["adapter_pda".as_bytes()],
        bump = args.adapter_pda_bump,
        seeds::program = args.adapter_program_id
    )]
    pub adapter_pda: Signer<'info>,

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
                args.integrator_program_id.to_bytes(),
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
/// This function allows an adapter to attest to a message. It performs the following steps:
/// 1. Checks if the adapter is enabled for receiving messages from the source chain.
/// 2. Initializes the attestation info account if it's newly created.
/// 3. Checks if the adapter has already attested to this message.
/// 4. Marks the adapter as having attested to the message.
/// 5. Increases the number of attested in `attestation_info`.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `args` - The arguments for the instruction, containing message details:
///   - `adapter_pda_bump`: The bump seed for the adapter's PDA
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
/// * The adapter is not enabled for receiving messages from the source chain
/// * The adapter has already attested to this message
///
/// # Returns
///
/// Returns `Ok(())` if the attestation is successful
///
/// # Events
///
/// Emits a `MessageAttestedTo` event
pub fn attest_message(ctx: Context<AttestMessage>, args: AttestMessageArgs) -> Result<()> {
    // Validate that the destination chain is this program's chain
    require!(
        args.dst_chain == CHAIN_ID,
        EndpointError::InvalidDestinationChain
    );

    let adapter_info = &ctx.accounts.adapter_info;
    let integrator_chain_config = &ctx.accounts.integrator_chain_config;
    let attestation_info = &mut ctx.accounts.attestation_info;

    // Check if the Adapter is an enabled receive Adapter for the Integrator and source chain
    require!(
        integrator_chain_config
            .recv_adapter_bitmap
            .get(adapter_info.index)
            .unwrap_or(false),
        EndpointError::AdapterNotEnabled
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
            args.integrator_program_id.to_bytes(),
            args.payload_hash,
        )?);
    }

    // Check if the Adapter has already attested
    require!(
        !attestation_info
            .attested_adapters
            .get(adapter_info.index)
            .unwrap_or(false),
        EndpointError::DuplicateMessageAttestation
    );

    // Mark the Adapter as having attested to the message
    attestation_info
        .attested_adapters
        .set(adapter_info.index, true)?;

    // Increment the number of attestations (saturates at 255)
    attestation_info.num_attested = attestation_info.num_attested.saturating_add(1);

    emit_cpi!(MessageAttestedTo {
        message_hash: attestation_info.message_hash,
        src_chain: args.src_chain,
        src_addr: UniversalAddress::from_bytes(args.src_addr),
        sequence: args.sequence,
        dst_chain: args.dst_chain,
        dst_addr: UniversalAddress::from_pubkey(&args.integrator_program_id),
        payload_hash: args.payload_hash,
        attested_bitmap: attestation_info.attested_adapters.as_value(),
        attesting_adapter: UniversalAddress::from_pubkey(&args.adapter_program_id),
    });

    Ok(())
}
