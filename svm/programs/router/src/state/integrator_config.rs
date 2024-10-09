use anchor_lang::prelude::*;

use crate::error::RouterError;

/// Manages the configuration for a specific integrator.
#[account]
#[derive(InitSpace)]
pub struct IntegratorConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Owner of the IntegratorConfig account
    pub owner: Pubkey,

    /// Program ID associated with this integrator
    pub program_id: Pubkey,

    /// Counter for assigning IDs to transceivers
    pub next_transceiver_id: u8,
}

impl IntegratorConfig {
    /// Seed prefix for deriving IntegratorConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_config";

    /// Maximum number of transceivers allowed
    pub const MAX_TRANSCEIVERS: u8 = 128;

    pub fn transfer_owner(&mut self, current_owner: &Signer, new_owner: Pubkey) -> Result<()> {
        require!(
            self.owner == current_owner.key(),
            RouterError::InvalidIntegratorAuthority
        );
        self.owner = new_owner;
        Ok(())
    }

    pub fn increment_transceiver_id(&mut self) -> Result<u8> {
        require!(
            self.next_transceiver_id < Self::MAX_TRANSCEIVERS,
            RouterError::MaxTransceiversReached
        );
        let current_id = self.next_transceiver_id;
        self.next_transceiver_id += 1;
        Ok(current_id)
    }
}
