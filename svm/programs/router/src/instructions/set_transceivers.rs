use crate::error::RouterError;
use crate::state::{IntegratorChainTransceivers, IntegratorConfig};
use crate::utils::bitmap::Bitmap;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(chain_id: u16)]
pub struct SetTransceivers<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        seeds = [IntegratorConfig::SEED_PREFIX, integrator_program.key().as_ref()],
        bump = integrator_config.bump,
        has_one = authority @ RouterError::InvalidIntegratorAuthority,
    )]
    pub integrator_config: Account<'info, IntegratorConfig>,

    #[account(
        mut,
        seeds = [
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program.key().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        bump = integrator_chain_transceivers.bump,
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,

    /// CHECK: This account is not read or written in this instruction
    pub integrator_program: UncheckedAccount<'info>,
}

pub fn set_in_transceivers(
    ctx: Context<SetTransceivers>,
    chain_id: u16,
    bitmap: u128,
) -> Result<()> {
    let integrator_chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;
    integrator_chain_transceivers.in_transceiver_bitmap = Bitmap::from_value(bitmap);

    msg!(
        "Incoming transceivers set successfully for chain ID: {}",
        chain_id
    );
    Ok(())
}

pub fn set_out_transceivers(
    ctx: Context<SetTransceivers>,
    chain_id: u16,
    bitmap: u128,
) -> Result<()> {
    let integrator_chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;
    integrator_chain_transceivers.out_transceiver_bitmap = Bitmap::from_value(bitmap);

    msg!(
        "Outgoing transceivers set successfully for chain ID: {}",
        chain_id
    );
    Ok(())
}