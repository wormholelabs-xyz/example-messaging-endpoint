#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use crate::instructions::register_transceiver::register_transceiver;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::{IntegratorConfig, RegisteredTransceiver};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_register_transceiver_success() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();

    // Initialize integrator config first
    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Now register a transceiver
    let transceiver_address = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator_program_id.as_ref(),
            &[0], // First transceiver ID
        ],
        &router::id(),
    );

    register_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_address,
    )
    .await
    .unwrap();

    // Fetch and verify the registered transceiver
    let registered_transceiver: RegisteredTransceiver =
        get_account(&mut context.banks_client, registered_transceiver_pda).await;

    assert_eq!(registered_transceiver.id, 0);
    assert_eq!(
        registered_transceiver.integrator_program_id,
        integrator_program_id
    );
    assert_eq!(registered_transceiver.address, transceiver_address);

    // Verify that the integrator config's next_transceiver_id has been incremented
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.next_transceiver_id, 1);
}

#[tokio::test]
async fn test_register_multiple_transceivers() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();

    // Initialize integrator config
    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    initialize_integrator_config(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Register two transceivers
    for i in 0..2 {
        let transceiver_address = Keypair::new().pubkey();
        let (registered_transceiver_pda, _) = Pubkey::find_program_address(
            &[
                RegisteredTransceiver::SEED_PREFIX,
                integrator_program_id.as_ref(),
                &[i],
            ],
            &router::id(),
        );

        register_transceiver(
            &mut context,
            &authority,
            &payer,
            integrator_config_pda,
            registered_transceiver_pda,
            integrator_program_id,
            transceiver_address,
        )
        .await
        .unwrap();

        // Verify the registered transceiver
        let registered_transceiver: RegisteredTransceiver =
            get_account(&mut context.banks_client, registered_transceiver_pda).await;
        assert_eq!(registered_transceiver.id, i);
        assert_eq!(
            registered_transceiver.integrator_program_id,
            integrator_program_id
        );
        assert_eq!(registered_transceiver.address, transceiver_address);
    }

    // Verify that the integrator config's next_transceiver_id has been incremented twice
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.next_transceiver_id, 2);
}
