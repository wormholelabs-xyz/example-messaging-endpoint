#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_transceiver::add_transceiver;
use crate::instructions::discard_admin::discard_admin;
use crate::instructions::register::register;
use crate::instructions::transfer_admin::transfer_admin;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::{IntegratorConfig, TransceiverInfo};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Pubkey, Pubkey) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();

    let (integrator_config_pda, _) = IntegratorConfig::pda(&mock_integrator::id());

    register(
        &mut context,
        &payer,
        &admin,
        integrator_config_pda,
        mock_integrator::id(),
    )
    .await
    .unwrap();

    (
        context,
        payer,
        admin,
        mock_integrator::id(),
        integrator_config_pda,
    )
}

async fn register_test_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config_pda: Pubkey,
    integrator_program_id: Pubkey,
) -> (Pubkey, Pubkey) {
    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    add_transceiver(
        context,
        admin,
        payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await
    .unwrap();

    (transceiver_program_id, registered_transceiver_pda)
}

#[tokio::test]
async fn test_add_transceiver_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let (transceiver_program_id, registered_transceiver_pda) = register_test_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Fetch and verify the registered transceiver
    let registered_transceiver: TransceiverInfo =
        get_account(&mut context.banks_client, registered_transceiver_pda).await;

    assert_eq!(registered_transceiver.index, 0);
    assert_eq!(
        registered_transceiver.integrator_program_id,
        integrator_program_id
    );
    assert_eq!(
        registered_transceiver.transceiver_program_id,
        transceiver_program_id
    );

    // Verify that the integrator config's transceivers list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.registered_transceivers.len(), 1);
    assert_eq!(
        integrator_config.registered_transceivers[0],
        transceiver_program_id
    );
}

#[tokio::test]
async fn test_register_multiple_transceivers() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register two transceivers
    let mut transceiver_program_ides = Vec::new();
    for id in 0..2 {
        let (transceiver_program_id, registered_transceiver_pda) = register_test_transceiver(
            &mut context,
            &admin,
            &payer,
            integrator_config_pda,
            integrator_program_id,
        )
        .await;
        transceiver_program_ides.push(transceiver_program_id);

        // Fetch and verify the registered transceiver
        let registered_transceiver: TransceiverInfo =
            get_account(&mut context.banks_client, registered_transceiver_pda).await;

        assert_eq!(registered_transceiver.index, id as u8);
        assert_eq!(
            registered_transceiver.integrator_program_id,
            integrator_program_id
        );
        assert_eq!(
            registered_transceiver.transceiver_program_id,
            transceiver_program_id
        );
    }

    // Verify that the integrator config's transceivers list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.registered_transceivers.len(), 2);
    assert_eq!(
        integrator_config.registered_transceivers,
        transceiver_program_ides
    );
}

#[tokio::test]
async fn test_register_max_transceivers() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register the maximum number of transceivers
    for _ in 0..IntegratorConfig::MAX_TRANSCEIVERS {
        register_test_transceiver(
            &mut context,
            &admin,
            &payer,
            integrator_config_pda,
            integrator_program_id,
        )
        .await;
    }

    // Attempt to register one more transceiver (should fail)
    let extra_transceiver_program_id = Keypair::new().pubkey();
    let (extra_registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &extra_transceiver_program_id);

    let result = add_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        extra_registered_transceiver_pda,
        integrator_program_id,
        extra_transceiver_program_id,
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
        integrator_config.registered_transceivers.len(),
        IntegratorConfig::MAX_TRANSCEIVERS
    );
}

#[tokio::test]
async fn test_add_transceiver_reinitialization() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register a transceiver
    let (transceiver_program_id, registered_transceiver_pda) = register_test_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Attempt to register the same transceiver again
    let result = add_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
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
    assert_eq!(integrator_config.registered_transceivers.len(), 1);
    assert_eq!(
        integrator_config.registered_transceivers[0],
        transceiver_program_id
    );
}

#[tokio::test]
async fn test_add_transceiver_non_authority() {
    let (mut context, payer, _, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Create a non-authority signer
    let non_authority = Keypair::new();

    // Attempt to register a transceiver with non-authority signer
    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    let result = add_transceiver(
        &mut context,
        &non_authority, // Use non-authority signer
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await;

    // Verify that the transaction failed with the CallerNotAuthorized error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );

    // Verify that the integrator config's transceivers list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.registered_transceivers.len(), 0);
}

#[tokio::test]
async fn test_add_transceiver_with_transfer_in_progress() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let pending_admin = Keypair::new();

    // First, initiate a transfer
    transfer_admin(
        &mut context,
        &admin,
        &pending_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Now try to add a transceiver
    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    let result = add_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::AdminTransferInProgress.into())
        )
    );

    // Verify that the integrator config hasn't changed
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, Some(admin.pubkey()));
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );
    assert_eq!(integrator_config.registered_transceivers.len(), 0);
}

#[tokio::test]
async fn test_add_transceiver_with_immutable_config() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // First, discard the admin to make the config immutable
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    // Now try to add a transceiver
    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    let result = add_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await;

    // The transaction should fail due to immutable config
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );

    // Verify that the integrator config's transceivers list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.registered_transceivers.len(), 0);
}
