use anchor_lang::prelude::*;

use crate::error::RouterError;

/// Manages the configuration for a specific integrator.
#[account]
#[derive(InitSpace)]
pub struct IntegratorConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Program ID associated with this integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Admin of the IntegratorConfig account
    pub admin: Pubkey,

    /// Pending admin of the IntegratorConfig account
    /// If this exists, any other admin related functions will not be authorised
    /// This must be null (in other words claim_admin will need to be called) before other ixs are
    /// enabled
    pub pending_admin: Option<Pubkey>,

    /// Vector of registered transceiver addresses
    #[max_len(128)]
    pub registered_transceivers: Vec<Pubkey>,

    /// A boolean to mark if config is immutable in other words admin is discarded
    pub is_immutable: bool,
}

impl IntegratorConfig {
    /// Seed prefix for deriving IntegratorConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_config";

    /// Maximum number of transceivers allowed
    pub const MAX_TRANSCEIVERS: usize = 128;

    pub fn pda(integrator_program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, integrator_program_id.as_ref()],
            &crate::ID,
        )
    }

    pub fn check_admin(&self, signer: &Signer) -> Result<()> {
        require!(
            !self.is_immutable && self.admin == signer.key(),
            RouterError::CallerNotAuthorized
        );
        require!(
            self.pending_admin.is_none(),
            RouterError::AdminTransferInProgress
        );
        Ok(())
    }

    pub fn update_admin(&mut self, new_admin: Pubkey) -> Result<()> {
        self.admin = new_admin;
        Ok(())
    }

    /// The `init` constraint in the add_transceiver instruction checks that the transceiver has not been added. If it is,
    /// `AccountAlreadyInUse` error will be thrown
    pub fn add_transceiver(&mut self, transceiver: Pubkey) -> Result<()> {
        require!(
            self.registered_transceivers.len() < Self::MAX_TRANSCEIVERS,
            RouterError::MaxTransceiversReached
        );
        self.registered_transceivers.push(transceiver);
        Ok(())
    }
}
