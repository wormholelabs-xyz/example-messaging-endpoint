#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::IntegratorConfig;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_initialize_integrator_config_success() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.program_test_context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator config
    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify the initialized account
    let integrator_config: IntegratorConfig =
        get_account(&mut context.program_test_context.banks_client, integrator_config_pda).await;

    assert_eq!(integrator_config.authority, authority.pubkey());
    assert_eq!(integrator_config.program_id, integrator_program_id);
    assert_eq!(integrator_config.next_transceiver_id, 0);
}

// TODO (@bingyuyap): this somehow fails, spent too much time on this. Will revisit
#[tokio::test]
async fn test_initialize_integrator_config_already_initialized() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.program_test_context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator config
    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Try to initialize again
    let result = initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Print debug information
    println!("Result of second initialization: {:?}", result);

    // Assert that the second initialization fails
    assert!(result.is_err(), "Expected an error, but got: {:?}", result);
}

#[tokio::test]
async fn test_initialize_integrator_config_different_programs() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.program_test_context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id_1 = Keypair::new().pubkey();
    let integrator_program_id_2 = Keypair::new().pubkey();

    let (integrator_config_pda_1, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id_1.as_ref(),
        ],
        &router::id(),
    );

    let (integrator_config_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id_2.as_ref(),
        ],
        &router::id(),
    );

    // Initialize for program 1
    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda_1,
        integrator_program_id_1,
    )
    .await
    .unwrap();

    // Initialize for program 2
    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda_2,
        integrator_program_id_2,
    )
    .await
    .unwrap();

    // Fetch and verify both accounts
    let integrator_config_1: IntegratorConfig =
        get_account(&mut context.program_test_context.banks_client, integrator_config_pda_1).await;
    let integrator_config_2: IntegratorConfig =
        get_account(&mut context.program_test_context.banks_client, integrator_config_pda_2).await;

    assert_eq!(integrator_config_1.program_id, integrator_program_id_1);
    assert_eq!(integrator_config_2.program_id, integrator_program_id_2);
}
