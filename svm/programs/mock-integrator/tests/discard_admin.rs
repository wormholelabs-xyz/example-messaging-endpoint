#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::discard_admin::discard_admin;
use crate::instructions::register::register;
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
async fn test_discard_admin_success() {
    let (mut context, payer, admin, _, integrator_config_pda) = setup_test_environment().await;

    let result = discard_admin(&mut context, &admin, &payer, integrator_config_pda).await;

    assert!(result.is_ok());

    // Verify that the admin has been discarded
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.is_immutable, true);
}

#[tokio::test]
async fn test_discard_admin_non_authority() {
    let (mut context, payer, _, _, integrator_config_pda) = setup_test_environment().await;

    let non_authority = Keypair::new();

    let result = discard_admin(&mut context, &non_authority, &payer, integrator_config_pda).await;

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
async fn test_discard_admin_already_discarded() {
    let (mut context, payer, admin, _, integrator_config_pda) = setup_test_environment().await;

    // First, discard the admin
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    // Try to discard the admin again
    let result = discard_admin(&mut context, &admin, &payer, integrator_config_pda).await;

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
async fn test_discard_admin_with_pending_transfer() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    // First, initiate a transfer
    crate::instructions::transfer_admin::transfer_admin(
        &mut context,
        &admin,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Now try to discard the admin
    let result = discard_admin(&mut context, &admin, &payer, integrator_config_pda).await;

    assert!(result.is_err());

    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::AdminTransferInProgress.into())
        )
    );
}
