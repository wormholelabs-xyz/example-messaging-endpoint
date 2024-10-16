use anchor_lang::prelude::*;

use crate::error::RouterError;

/// Manages the configuration for a specific integrator.
#[account]
#[derive(InitSpace)]
pub struct IntegratorConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Admin of the IntegratorConfig account
    pub admin: Pubkey,

    /// Program ID associated with this integrator
    pub integrator_program_id: Pubkey,

    /// Vector of registered transceiver addresses
    #[max_len(128)]
    pub registered_transceivers: Vec<Pubkey>,
}

impl IntegratorConfig {
    /// Seed prefix for deriving IntegratorConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_config";

    /// Maximum number of transceivers allowed
    pub const MAX_TRANSCEIVERS: usize = 128;

    pub fn pda(integrator_program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                integrator_program_id.as_ref(),
            ],
            &crate::ID,
        )
    }

    pub fn update_admin(&mut self, current_admin: &Signer, new_admin: Pubkey) -> Result<()> {
        require!(
            self.admin == current_admin.key(),
            RouterError::InvalidIntegratorAuthority
        );
        self.admin = new_admin;
        Ok(())
    }

    pub fn add_transceiver(&mut self, transceiver: Pubkey) -> Result<()> {
        require!(
            self.registered_transceivers.len() < Self::MAX_TRANSCEIVERS,
            RouterError::MaxTransceiversReached
        );
        self.registered_transceivers.push(transceiver);
        Ok(())
    }
}
