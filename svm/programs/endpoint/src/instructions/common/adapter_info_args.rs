use anchor_lang::prelude::*;

/// Common arguments for adapter info operations
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AdapterInfoArgs {
    /// The ID of the chain
    pub chain_id: u16,

    /// The Pubkey of the adapter
    pub adapter_program_id: Pubkey,

    /// The Pubkey of the integrator program
    pub integrator_program_id: Pubkey,
}
