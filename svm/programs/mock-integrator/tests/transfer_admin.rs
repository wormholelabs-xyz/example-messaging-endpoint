#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::discard_admin::discard_admin;
use crate::instructions::register::register;
use crate::instructions::transfer_admin::{claim_admin, transfer_admin};
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::{error::RouterError, state::IntegratorConfig};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Pubkey, Pubkey) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);

    register(
        &mut context,
        &payer,
        &admin,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    (
        context,
        payer,
        admin,
        integrator_program_id,
        integrator_config_pda,
    )
}

#[tokio::test]
async fn test_transfer_admin_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    // Verify that there's no pending transfer initially
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, None);

    let result = transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the pending_admin has been set
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, Some(new_admin.pubkey()));
    assert_eq!(integrator_config.admin, admin.pubkey()); // Admin should not change yet
}

#[tokio::test]
async fn test_transfer_admin_with_pending_transfer() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin1 = Keypair::new();
    let new_admin2 = Keypair::new();

    // First transfer
    transfer_admin(
        &mut context,
        &admin,
        &new_admin1.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Attempt second transfer
    let result = transfer_admin(
        &mut context,
        &admin,
        &new_admin2.pubkey(),
        &payer,
        integrator_config_pda,
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

    // Verify that the pending_admin is still the first new admin
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, Some(new_admin1.pubkey()));
}

#[tokio::test]
async fn test_cancel_admin_transfer() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    // Initiate transfer
    transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Cancel transfer by claiming as current admin
    let result = claim_admin(&mut context, &admin, &payer, integrator_config_pda).await;

    assert!(result.is_ok());

    // Verify that the pending_admin has been cleared and admin remains unchanged
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, None);
    assert_eq!(integrator_config.admin, admin.pubkey());
}

#[tokio::test]
async fn test_claim_admin_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    // First, transfer admin
    transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Verify that the pending_admin is set
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, Some(new_admin.pubkey()));

    // Now, claim admin
    let result = claim_admin(&mut context, &new_admin, &payer, integrator_config_pda).await;

    assert!(result.is_ok());

    // Verify that the admin has been updated and pending_admin is cleared
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, new_admin.pubkey());
    assert_eq!(integrator_config.pending_admin, None);
}

#[tokio::test]
async fn test_cancel_claim_admin_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    // First, transfer admin
    transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Verify that the pending_admin is set
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.pending_admin, Some(new_admin.pubkey()));

    // Now, claim admin
    let result = claim_admin(&mut context, &admin, &payer, integrator_config_pda).await;

    assert!(result.is_ok());

    // Verify that the admin has been updated and pending_admin is cleared
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
    assert_eq!(integrator_config.pending_admin, None);
}

#[tokio::test]
async fn test_claim_admin_no_pending_admin() {
    let (mut context, payer, admin, _, integrator_config_pda) = setup_test_environment().await;

    let random_user = Keypair::new();

    // Attempt to claim admin with a random user (neither admin nor pending_admin)
    let result = claim_admin(&mut context, &random_user, &payer, integrator_config_pda).await;

    // Assert that the operation fails with CallerNotAuthorized error
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );

    // Verify that the admin remains unchanged
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
    assert_eq!(integrator_config.pending_admin, None);
}

#[tokio::test]
async fn test_transfer_admin_non_authority() {
    let (mut context, payer, _, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let non_authority = Keypair::new();
    let new_admin = Keypair::new();

    let result = transfer_admin(
        &mut context,
        &non_authority,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
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
}

#[tokio::test]
async fn test_claim_admin_unauthorized() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();
    let unauthorized = Keypair::new();

    // First, transfer admin
    transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Now, try to claim admin with an unauthorized key
    let result = claim_admin(&mut context, &unauthorized, &payer, integrator_config_pda).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::CallerNotAuthorized.into())
        )
    );
}

#[tokio::test]
async fn test_transfer_admin_with_transfer_in_progress() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let pending_admin = Keypair::new();
    let new_admin = Keypair::new();

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

    // Now try to initiate another transfer
    let result = transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
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

    // Verify that the admin and pending_admin haven't changed
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );
}

#[tokio::test]
async fn test_transfer_admin_with_immutable_config() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Discard the admin to make the config immutable
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    let new_admin = Keypair::new();

    // Now try to transfer admin
    let result = transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
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

    // Verify that the integrator config is immutable
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.is_immutable, true);
}
