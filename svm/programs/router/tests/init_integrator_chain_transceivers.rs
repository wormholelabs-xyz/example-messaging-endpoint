#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::init_integrator_chain_transceivers::init_integrator_chain_transceivers;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::IntegratorChainTransceivers;
use router::utils::bitmap::Bitmap;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_init_integrator_chain_transceivers_success() {
    // Set up the test environment
    let (mut context, config_pda) = setup().await;
    let payer = context.payer.insecure_clone();
    let owner = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id: u16 = 1;

    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator chain transceivers
    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &owner,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify the initialized account
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;

    assert_eq!(integrator_chain_transceivers.chain_id, chain_id);
    assert_eq!(integrator_chain_transceivers.owner, owner.pubkey());
    assert_eq!(integrator_chain_transceivers.next_in_transceiver_id, 0);
    assert_eq!(integrator_chain_transceivers.next_out_transceiver_id, 0);
    assert_eq!(
        integrator_chain_transceivers.in_transceiver_bitmap,
        Bitmap::new()
    );
    assert_eq!(
        integrator_chain_transceivers.out_transceiver_bitmap,
        Bitmap::new()
    );
}

#[tokio::test]
async fn test_init_integrator_chain_transceivers_already_initialized() {
    // Set up the test environment
    let (mut context, config_pda) = setup().await;
    let payer = context.payer.insecure_clone();
    let owner = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id: u16 = 1;

    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator chain transceivers
    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &owner,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Try to initialize again
    let result = init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &owner,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await;

    // Assert that the second initialization fails
    assert!(result.is_err());
}

#[tokio::test]
async fn test_init_integrator_chain_transceivers_different_chains() {
    // Set up the test environment
    let (mut context, config_pda) = setup().await;
    let payer = context.payer.insecure_clone();
    let owner = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id_1: u16 = 1;
    let chain_id_2: u16 = 2;

    let (integrator_chain_transceivers_pda_1, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_1.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    let (integrator_chain_transceivers_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_2.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize for chain 1
    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &owner,
        &payer,
        integrator_chain_transceivers_pda_1,
        chain_id_1,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Initialize for chain 2
    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &owner,
        &payer,
        integrator_chain_transceivers_pda_2,
        chain_id_2,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify both accounts
    let integrator_chain_transceivers_1: IntegratorChainTransceivers = get_account(
        &mut context.banks_client,
        integrator_chain_transceivers_pda_1,
    )
    .await;
    let integrator_chain_transceivers_2: IntegratorChainTransceivers = get_account(
        &mut context.banks_client,
        integrator_chain_transceivers_pda_2,
    )
    .await;

    assert_eq!(integrator_chain_transceivers_1.chain_id, chain_id_1);
    assert_eq!(integrator_chain_transceivers_2.chain_id, chain_id_2);
}
