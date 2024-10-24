use anchor_lang::prelude::*;

/// Common arguments for transceiver info operations
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransceiverInfoArgs {
    /// The ID of the chain
    pub chain_id: u16,

    /// The Pubkey of the transceiver
    pub transceiver_program_id: Pubkey,

    /// The Pubkey of the integrator program
    pub integrator_program_id: Pubkey,
}
