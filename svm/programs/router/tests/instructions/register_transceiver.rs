use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::RegisterTransceiver;
use router::instructions::TransceiverType; // Add this import
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::common::setup::TestContext;

pub async fn register_transceiver(
    context: &mut TestContext,
    config_pda: Pubkey,
    integrator: Pubkey,
    authority: &Keypair,
    registered_transceiver: Pubkey,
    integrator_chain_transceivers: Pubkey,
    chain_id: u16,
    transceiver_type: TransceiverType, // Add this parameter
    transceiver_address: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = RegisterTransceiver {
        config: config_pda,
        integrator,
        authority: authority.pubkey(),
        payer: context.payer.pubkey(),
        registered_transceiver,
        integrator_chain_transceivers,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::RegisterTransceiver {
            chain_id,
            transceiver_type, // Add this field
            transceiver_address,
        }
        .data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}

// Helper function to get the PDA for a registered transceiver
pub fn get_registered_transceiver_pda(
    integrator_id: u64,
    chain_id: u16,
    transceiver_id: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            router::state::RegisteredTransceiver::SEED_PREFIX,
            &integrator_id.to_le_bytes(),
            &chain_id.to_le_bytes(),
            &transceiver_id.to_le_bytes(),
        ],
        &router::id(),
    )
}

// Helper function to get the PDA for integrator chain transceivers
pub fn get_integrator_chain_transceivers_pda(integrator_id: u64, chain_id: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            router::state::IntegratorChainTransceivers::SEED_PREFIX,
            &integrator_id.to_le_bytes(),
            &chain_id.to_le_bytes(),
        ],
        &router::id(),
    )
}