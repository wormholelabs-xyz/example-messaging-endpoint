#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::discard_admin::discard_admin;
use crate::instructions::register::register;
use crate::instructions::transfer_admin::transfer_admin;
use crate::instructions::update_admin::update_admin;
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
async fn test_update_admin_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    let result = update_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the admin has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, new_admin.pubkey());
}

#[tokio::test]
async fn test_update_admin_non_authority() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let non_authority = Keypair::new();
    let new_admin = Keypair::new();

    let result = update_admin(
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

    // Verify that the admin has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
}

#[tokio::test]
async fn test_update_admin_same_address() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let result = update_admin(
        &mut context,
        &admin,
        &admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the admin remains the same
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
}

#[tokio::test]
async fn test_update_admin_with_transfer_in_progress() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let pending_admin = Keypair::new();
    let new_admin = Keypair::new();

    // Initiate a transfer
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

    // Verify that the pending_admin is set
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );

    // Try to update admin
    let result = update_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Check that the update fails due to transfer in progress
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::AdminTransferInProgress.into())
        )
    );

    // Verify that the admin and pending_admin remain unchanged
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, admin.pubkey());
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );
}

#[tokio::test]
async fn test_update_admin_with_immutable_config() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Discard the admin to make the config immutable
    crate::instructions::discard_admin::discard_admin(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
    )
    .await
    .unwrap();

    let new_admin = Keypair::new();

    // Now try to update admin
    let result = update_admin(
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
