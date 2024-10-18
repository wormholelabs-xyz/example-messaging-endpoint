#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_transceiver::add_transceiver;
use crate::instructions::disable_transceiver::{
    disable_recv_transceiver, disable_send_transceiver,
};
use crate::instructions::discard_admin::discard_admin;
use crate::instructions::enable_transceiver::{enable_recv_transceiver, enable_send_transceiver};
use crate::instructions::register::register;
use crate::instructions::transfer_admin::transfer_admin;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::{
    state::{IntegratorChainConfig, IntegratorConfig, TransceiverInfo},
    utils::bitmap::Bitmap,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn initialize_test_environment(
    context: &mut ProgramTestContext,
) -> (Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, u16) {
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);

    register(
        context,
        &payer,
        &admin,
        integrator_config_pda,
        mock_integrator::id(),
    )
    .await
    .unwrap();

    // Prepare integrator_chain_config_pda
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);

    // Register a transceiver
    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    add_transceiver(
        context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await
    .unwrap();

    (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver_program_id,
        chain_id,
    )
}

async fn verify_transceiver_state(
    context: &mut ProgramTestContext,
    integrator_chain_config_pda: Pubkey,
    expected_recv_bitmap: u128,
    expected_send_bitmap: u128,
) {
    let integrator_chain_config: IntegratorChainConfig =
        get_account(&mut context.banks_client, integrator_chain_config_pda).await;

    assert_eq!(
        integrator_chain_config.recv_transceiver_bitmap,
        Bitmap::from_value(expected_recv_bitmap)
    );
    assert_eq!(
        integrator_chain_config.send_transceiver_bitmap,
        Bitmap::from_value(expected_send_bitmap)
    );
}

#[tokio::test]
async fn test_disable_recv_transceiver_success() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Set the receive transceiver first
    enable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Disable the receive transceiver
    let result = disable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the transceiver is disabled
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0, 0).await;
}

#[tokio::test]
async fn test_disable_send_transceiver_success() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Set the send transceiver first
    enable_send_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Verify that the transceiver is disabled
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0, 1).await;

    // Disable the send transceiver
    let result = disable_send_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the transceiver is disabled
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0, 0).await;
}

#[tokio::test]
async fn test_disable_transceiver_invalid_authority() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Set the receive transceiver first
    enable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Create a new keypair to act as an invalid authority
    let invalid_authority = Keypair::new();

    let result = disable_recv_transceiver(
        &mut context,
        &invalid_authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to invalid authority
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );
}

#[tokio::test]
async fn test_disable_transceiver_invalid_transceiver_id() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        _transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Use an invalid transceiver pubkey
    let invalid_transceiver = Keypair::new().pubkey();
    let payer = context.payer.insecure_clone();

    let result = disable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        invalid_transceiver,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to invalid transceiver id
    // It will return AccountNotInitialized because the transceiver is not registered
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(3012))
    );
}

#[tokio::test]
async fn test_disable_already_disabled_transceiver() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Set the receive transceiver first to make sure that the integrator_chain_config_pda is
    // initialized
    enable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Disable the receive transceiver
    let result = disable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the transceiver is disabled
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0, 0).await;

    // Try to disable the already disabled receive transceiver
    let result = disable_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to the transceiver already being disabled
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::TransceiverAlreadyDisabled.into())
        )
    );
}

#[tokio::test]
async fn test_disable_transceiver_with_transfer_in_progress() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();
    let pending_admin = Keypair::new();

    // Enable the receive transceiver first
    enable_recv_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Initiate an admin transfer
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

    // Now try to disable the transceiver
    let result = disable_recv_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
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

    // Verify that the transceiver state hasn't changed
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 1, 0).await;

    // Verify that the integrator config hasn't changed
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );
}

#[tokio::test]
async fn test_disable_transceiver_with_immutable_config() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Enable the receive transceiver first
    enable_recv_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Discard the admin to make the config immutable
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    // Now try to disable the transceiver
    let result = disable_recv_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver,
        integrator_program_id,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );

    // Verify that the transceiver state hasn't changed
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 1, 0).await;

    // Verify that the integrator config is immutable
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.is_immutable, true);
}
