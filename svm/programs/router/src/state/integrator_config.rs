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
    pub integrator_program_id: Pubkey,

    /// Vector of registered transceiver addresses
    #[max_len(32)]
    pub transceivers: Vec<Pubkey>,
}

impl IntegratorConfig {
    /// Seed prefix for deriving IntegratorConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_config";

    /// Maximum number of transceivers allowed
    pub const MAX_TRANSCEIVERS: usize = 128;

    pub fn transfer_owner(&mut self, current_owner: &Signer, new_owner: Pubkey) -> Result<()> {
        require!(
            self.owner == current_owner.key(),
            RouterError::InvalidIntegratorAuthority
        );
        self.owner = new_owner;
        Ok(())
    }

    pub fn add_transceiver(&mut self, transceiver: Pubkey) -> Result<()> {
        require!(
            self.transceivers.len() < Self::MAX_TRANSCEIVERS,
            RouterError::MaxTransceiversReached
        );
        self.transceivers.push(transceiver);
        Ok(())
    }
}
