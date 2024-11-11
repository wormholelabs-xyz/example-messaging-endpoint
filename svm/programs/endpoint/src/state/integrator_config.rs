use anchor_lang::prelude::*;

use crate::error::EndpointError;

/// Manages the configuration for a specific integrator.
#[account]
#[derive(InitSpace, Debug)]
pub struct IntegratorConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Program ID associated with this integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// Admin of the IntegratorConfig account
    pub admin: Option<Pubkey>,

    /// Pending admin of the IntegratorConfig account
    /// If this exists, any other admin related functions will not be authorised
    /// This must be null (in other words claim_admin will need to be called) before other ixs are
    /// enabled
    pub pending_admin: Option<Pubkey>,

    /// Vector of registered adapter addresses
    #[max_len(128)]
    pub adapter_infos: Vec<Pubkey>,
}

impl IntegratorConfig {
    /// Seed prefix for deriving IntegratorConfig PDAs
    pub const SEED_PREFIX: &'static [u8] = b"integrator_config";

    /// Maximum number of adapters allowed
    pub const MAX_ADAPTERS: usize = 128;

    pub fn pda(integrator_program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, integrator_program_id.as_ref()],
            &crate::ID,
        )
    }

    pub fn check_admin(&self, signer: &Signer) -> Result<()> {
        require!(
            self.admin == Some(signer.key()),
            EndpointError::CallerNotAuthorized
        );
        require!(
            self.pending_admin.is_none(),
            EndpointError::AdminTransferInProgress
        );
        Ok(())
    }

    /// The `init` constraint in the add_adapter instruction checks that the adapter has not been added. If it is,
    /// `AccountAlreadyInUse` error will be thrown
    pub fn add_adapter(&mut self, adapter: Pubkey) -> Result<()> {
        require!(
            self.adapter_infos.len() < Self::MAX_ADAPTERS,
            EndpointError::MaxAdaptersReached
        );
        self.adapter_infos.push(adapter);
        Ok(())
    }
}
