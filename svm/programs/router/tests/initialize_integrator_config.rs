#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::register::register;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::IntegratorConfig;
use solana_program_test::*;

use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

#[tokio::test]
async fn test_register_success() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new().pubkey();
    let integrator_program = Keypair::new();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator config
    register(
        &mut context,
        &payer,
        authority,
        integrator_config_pda,
        &integrator_program,
    )
    .await
    .unwrap();

    // Fetch and verify the initialized account
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;

    assert_eq!(integrator_config.admin, authority);
    assert_eq!(
        integrator_config.integrator_program_id,
        integrator_program.pubkey()
    );
    assert_eq!(integrator_config.registered_transceivers.len(), 0);
}

#[tokio::test]
async fn test_register_reinitialization() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new().pubkey();
    let integrator_program = Keypair::new();

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator config
    register(
        &mut context,
        &payer,
        authority,
        integrator_config_pda,
        &integrator_program,
    )
    .await
    .unwrap();

    // Try to initialize again
    let result = register(
        &mut context,
        &payer,
        authority,
        integrator_config_pda,
        &integrator_program,
    )
    .await;

    let err = result.unwrap_err();
    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(SystemError::AccountAlreadyInUse as u32)
        ),
        "Expected AccountAlreadyInUse error, but got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_register_different_programs() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new().pubkey();
    let integrator_program_1 = Keypair::new();
    let integrator_program_2 = Keypair::new();

    let (integrator_config_pda_1, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_1.pubkey().as_ref(),
        ],
        &router::id(),
    );

    let (integrator_config_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_2.pubkey().as_ref(),
        ],
        &router::id(),
    );

    // Initialize for program 1
    register(
        &mut context,
        &payer,
        authority,
        integrator_config_pda_1,
        &integrator_program_1,
    )
    .await
    .unwrap();

    // Initialize for program 2
    register(
        &mut context,
        &payer,
        authority,
        integrator_config_pda_2,
        &integrator_program_2,
    )
    .await
    .unwrap();

    // Fetch and verify both accounts
    let integrator_config_1: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda_1).await;
    let integrator_config_2: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda_2).await;

    assert_eq!(
        integrator_config_1.integrator_program_id,
        integrator_program_1.pubkey()
    );
    assert_eq!(
        integrator_config_2.integrator_program_id,
        integrator_program_2.pubkey()
    );
}
