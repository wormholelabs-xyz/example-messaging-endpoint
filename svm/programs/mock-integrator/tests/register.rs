#![cfg(feature = "test-sbf")]

use router::state::{IntegratorConfig, SequenceTracker};
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

    let (integrator_config, _) = IntegratorConfig::pda(&mock_integrator::id());

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

    assert_eq!(integrator_config_data.admin, Some(admin.pubkey()));
    assert_eq!(
        integrator_config_data.integrator_program_id,
        mock_integrator::id()
    );
    assert!(integrator_config_data.transceiver_infos.is_empty());

    let (sequence_tracker, _) = SequenceTracker::pda(&mock_integrator::id());
    let sequence_tracker_data: router::state::SequenceTracker =
        get_account(&mut context.banks_client, sequence_tracker).await;
    // Verify that the integrator program ID and sequence are correct
    assert_eq!(
        sequence_tracker_data.integrator_program_id,
        mock_integrator::id(),
        "Integrator program ID does not match"
    );

    let expected_sequence = 0;
    assert_eq!(
        sequence_tracker_data.sequence, expected_sequence,
        "Sequence number is incorrect"
    );
}

#[tokio::test]
async fn test_invoke_register_reinitialization() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();

    let (integrator_config, _) = IntegratorConfig::pda(&mock_integrator::id());

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
