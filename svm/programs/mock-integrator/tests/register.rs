#![cfg(feature = "test-sbf")]

use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

mod common;
mod instructions;

use crate::common::setup::{get_account, setup};
use instructions::register::register;

#[tokio::test]
async fn test_invoke_register() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();

    let (integrator_config, _) = Pubkey::find_program_address(
        &[
            router::state::IntegratorConfig::SEED_PREFIX,
            mock_integrator::id().as_ref(),
        ],
        &router::id(),
    );

    let result = register(
        &mut context,
        &payer,
        &admin,
        integrator_config,
        mock_integrator::id(),
    )
    .await;

    // Assert that the transaction was successful
    assert!(
        result.is_ok(),
        "Failed to process transaction: {:?}",
        result
    );

    // Verify that the integrator config account was created and initialized correctly
    let integrator_config_data: router::state::IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config).await;

    assert_eq!(integrator_config_data.admin, admin.pubkey());
    assert_eq!(
        integrator_config_data.integrator_program_id,
        mock_integrator::id()
    );
    assert!(integrator_config_data.registered_transceivers.is_empty());
}

#[tokio::test]
async fn test_invoke_register_reinitialization() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();

    let (integrator_config, _) = Pubkey::find_program_address(
        &[
            router::state::IntegratorConfig::SEED_PREFIX,
            mock_integrator::id().as_ref(),
        ],
        &router::id(),
    );

    // First registration
    let result = register(
        &mut context,
        &payer,
        &admin,
        integrator_config,
        mock_integrator::id(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Failed to process first registration: {:?}",
        result
    );

    // Attempt to register again
    let result = register(
        &mut context,
        &payer,
        &admin,
        integrator_config,
        mock_integrator::id(),
    )
    .await;

    // Assert that the second registration fails
    assert!(
        result.is_err(),
        "Second registration should have failed but succeeded"
    );

    // Check for the specific error
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(SystemError::AccountAlreadyInUse as u32)
        ),
        "Unexpected error on reinitialization attempt"
    );
}
