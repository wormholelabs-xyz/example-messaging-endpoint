use crate::state::{Config, IntegratorChainTransceivers, RegisteredTransceiver};
use anchor_lang::prelude::*;

/// Enum representing the type of transceiver being registered
#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TransceiverType {
    /// Incoming transceiver
    In,
    /// Outgoing transceiver
    Out,
}

/// Accounts struct for registering a new transceiver
#[derive(Accounts)]
#[instruction(chain_id: u16, transceiver_type: TransceiverType)]
pub struct RegisterTransceiver<'info> {
    /// The global configuration account
    #[account(
        seeds = [Config::SEED_PREFIX],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// The Integrator program account for which the transceiver is being registered
    pub integrator: AccountInfo<'info>,

    /// The authority of the Integrator
    pub authority: Signer<'info>,

    /// The account paying for the registration
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The IntegratorChainTransceivers account for the specific chain
    #[account(
        mut,
        seeds = [
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator.key().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub integrator_chain_transceivers: Account<'info, IntegratorChainTransceivers>,

    /// The RegisteredTransceiver account being initialized
    #[account(
        init,
        payer = payer,
        space = 8 + RegisteredTransceiver::INIT_SPACE,
        seeds = [
            RegisteredTransceiver::SEED_PREFIX,
            integrator.key().as_ref(),
            chain_id.to_le_bytes().as_ref(),
            {
                let transceiver_id = match transceiver_type {
                    TransceiverType::In => integrator_chain_transceivers.next_in_transceiver_id,
                    TransceiverType::Out => integrator_chain_transceivers.next_out_transceiver_id,
                };
                transceiver_id.to_le_bytes().as_ref()
            }
        ],
        bump
    )]
    pub registered_transceiver: Account<'info, RegisteredTransceiver>,

    /// The System Program
    pub system_program: Program<'info, System>,
}

/// Registers a new transceiver for a specific integrator and chain
///
/// This function creates a new RegisteredTransceiver account and updates the
/// IntegratorChainTransceivers account to reflect the new transceiver.
///
/// # Arguments
///
/// * `ctx` - The context of the instruction, containing the accounts
/// * `chain_id` - The ID of the chain for which the transceiver is being registered
/// * `transceiver_type` - The type of the transceiver (In or Out)
/// * `transceiver_address` - The public key of the transceiver address
///
/// # Returns
///
/// Returns `Ok(())` if the registration is successful
pub fn register_transceiver(
    ctx: Context<RegisterTransceiver>,
    chain_id: u16,
    transceiver_type: TransceiverType,
    transceiver_address: Pubkey,
) -> Result<()> {
    let chain_transceivers = &mut ctx.accounts.integrator_chain_transceivers;

    let transceiver_id = match transceiver_type {
        TransceiverType::In => chain_transceivers.next_in_transceiver_id,
        TransceiverType::Out => chain_transceivers.next_out_transceiver_id,
    };

    // Update the bitmap
    // `set_in_transceiver` and `set_out_transceiver` returns `BitmapIndexOutOfBounds` if it
    // exceeds [`Bitmap::BITS`].
    match transceiver_type {
        TransceiverType::In => {
            chain_transceivers.set_in_transceiver(transceiver_id as u8, true)?;
            chain_transceivers.next_in_transceiver_id += 1;
        }
        TransceiverType::Out => {
            chain_transceivers.set_out_transceiver(transceiver_id as u8, true)?;
            chain_transceivers.next_out_transceiver_id += 1;
        }
    }

    // Initialize the RegisteredTransceiver account
    let registered_transceiver = &mut ctx.accounts.registered_transceiver;
    registered_transceiver.id = transceiver_id;
    registered_transceiver.chain_id = chain_id;
    registered_transceiver.address = transceiver_address;

    Ok(())
}
