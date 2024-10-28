#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MessageArgs {
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: MessageArgs)]
pub struct Message<'info> {
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
