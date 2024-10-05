use crate::{error::RouterError, state::IntegratorChainTransceivers};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferIntegratorChainTransceiversOwnership<'info> {
    /// The current owner of the IntegratorChainTransceivers account
    pub owner: Signer<'info>,

    /// The IntegratorChainTransceivers account being transferred
    #[account(
        mut,
        has_one = owner @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,
}

pub fn transfer_integrator_chain_transceivers_ownership(
    ctx: Context<TransferIntegratorChainTransceiversOwnership>,
    new_owner: Pubkey,
) -> Result<()> {
    ctx.accounts
        .integrator_chain_transceivers
        .transfer_ownership(&ctx.accounts.owner, new_owner)
}
