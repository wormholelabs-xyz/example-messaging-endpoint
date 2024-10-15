#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::register::register;
use crate::instructions::update_admin::update_admin;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::IntegratorConfig;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Pubkey, Pubkey) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let owner = Keypair::new();
    let integrator_program = mock_integrator::id();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program.as_ref(),
        ],
        &router::id(),
    );

    register(
        &mut context,
        &payer,
        &owner,
        integrator_config_pda,
        integrator_program,
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

#[tokio::test]
async fn test_update_admin_success() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    let new_admin = Keypair::new();

    let result = update_admin(
        &mut context,
        &owner,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program,
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
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    let non_authority = Keypair::new();
    let new_admin = Keypair::new();

    let result = update_admin(
        &mut context,
        &non_authority,
        &new_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
        )
    );

    // Verify that the admin has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, owner.pubkey());
}

#[tokio::test]
async fn test_update_admin_same_address() {
    let (mut context, payer, owner, integrator_program, integrator_config_pda) =
        setup_test_environment().await;

    let result = update_admin(
        &mut context,
        &owner,
        &owner.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program,
    )
    .await;

    assert!(result.is_ok());

    // Verify that the admin remains the same
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, owner.pubkey());
}
