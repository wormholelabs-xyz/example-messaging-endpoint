#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use crate::instructions::register_transceiver::register_transceiver;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::{IntegratorConfig, RegisteredTransceiver};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Keypair, Pubkey) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let owner = Keypair::new();
    let integrator_program = Keypair::new();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
        ],
        &router::id(),
    );

    initialize_integrator_config(
        &mut context,
        &payer,
        owner.pubkey(),
        integrator_config_pda,
        &integrator_program,
    )
    .await
    .unwrap();

    (
        context,
        payer,
        owner,
        integrator_program,
        integrator_config_pda,
    )
}

async fn register_test_transceiver(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config_pda: Pubkey,
    integrator_program: &Keypair,
) -> (Pubkey, Pubkey) {
    let transceiver_address = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        RegisteredTransceiver::pda(&integrator_program.pubkey(), &transceiver_address);

    register_transceiver(
        context,
        owner,
        payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program.pubkey(),
        transceiver_address,
    )
    .await
    .unwrap();

    (transceiver_address, registered_transceiver_pda)
}

#[tokio::test]
async fn test_register_transceiver_success() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    let (transceiver_address, registered_transceiver_pda) = register_test_transceiver(
        &mut context,
        &owner,
        &payer,
        integrator_config_pda,
        &integrator_program,
    )
    .await;

    // Fetch and verify the registered transceiver
    let registered_transceiver: RegisteredTransceiver =
        get_account(&mut context.banks_client, registered_transceiver_pda).await;

    assert_eq!(registered_transceiver.id, 0);
    assert_eq!(
        registered_transceiver.integrator_program_id,
        integrator_program.pubkey()
    );
    assert_eq!(registered_transceiver.transceiver_address, transceiver_address);

    // Verify that the integrator config's transceivers list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.transceivers.len(), 1);
    assert_eq!(integrator_config.transceivers[0], transceiver_address);
}

#[tokio::test]
async fn test_register_multiple_transceivers() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    // Register two transceivers
    let mut transceiver_addresses = Vec::new();
    for _ in 0..2 {
        let (transceiver_address, _) = register_test_transceiver(
            &mut context,
            &owner,
            &payer,
            integrator_config_pda,
            &integrator_program,
        )
        .await;
        transceiver_addresses.push(transceiver_address);
    }

    // Verify that the integrator config's transceivers list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.transceivers.len(), 2);
    assert_eq!(integrator_config.transceivers, transceiver_addresses);
}

#[tokio::test]
async fn test_register_max_transceivers() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    // Register the maximum number of transceivers
    for _ in 0..IntegratorConfig::MAX_TRANSCEIVERS {
        register_test_transceiver(
            &mut context,
            &owner,
            &payer,
            integrator_config_pda,
            &integrator_program,
        )
        .await;
    }

    // Attempt to register one more transceiver (should fail)
    let extra_transceiver_address = Keypair::new().pubkey();
    let (extra_registered_transceiver_pda, _) =
        RegisteredTransceiver::pda(&integrator_program.pubkey(), &extra_transceiver_address);

    let result = register_transceiver(
        &mut context,
        &owner,
        &payer,
        integrator_config_pda,
        extra_registered_transceiver_pda,
        integrator_program.pubkey(),
        extra_transceiver_address,
    )
    .await;

    // Verify that the transaction failed with the MaxTransceiversReached error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::MaxTransceiversReached.into())
        )
    );

    // Verify that the integrator config's transceivers list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(
        integrator_config.transceivers.len(),
        IntegratorConfig::MAX_TRANSCEIVERS
    );
}

#[tokio::test]
async fn test_register_transceiver_reinitialization() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    // Register a transceiver
    let (transceiver_address, registered_transceiver_pda) = register_test_transceiver(
        &mut context,
        &owner,
        &payer,
        integrator_config_pda,
        &integrator_program,
    )
    .await;

    // Attempt to register the same transceiver again
    let result = register_transceiver(
        &mut context,
        &owner,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program.pubkey(),
        transceiver_address,
    )
    .await;

    // Verify that the transaction failed with the appropriate error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(SystemError::AccountAlreadyInUse as u32)
        ),
    );

    // Verify that the integrator config's transceivers list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.transceivers.len(), 1);
    assert_eq!(integrator_config.transceivers[0], transceiver_address);
}

#[tokio::test]
async fn test_register_transceiver_non_authority() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    // Create a non-authority signer
    let non_authority = Keypair::new();

    // Attempt to register a transceiver with non-authority signer
    let transceiver_address = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        RegisteredTransceiver::pda(&integrator_program.pubkey(), &transceiver_address);

    let result = register_transceiver(
        &mut context,
        &non_authority, // Use non-authority signer
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program.pubkey(),
        transceiver_address,
    )
    .await;

    // Verify that the transaction failed with the InvalidIntegratorAuthority error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
        )
    );

    // Verify that the integrator config's transceivers list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.transceivers.len(), 0);
}
