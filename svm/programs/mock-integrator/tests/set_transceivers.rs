#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::register::register;
use crate::instructions::register_transceiver::register_transceiver;
use crate::instructions::set_transceivers::{set_recv_transceiver, set_send_transceiver};

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
    let integrator_program = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&mock_integrator::id());

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
        IntegratorChainConfig::pda(&integrator_program, chain_id);

    // Register a transceiver
    let transceiver_address = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program, &transceiver_address);

    register_transceiver(
        context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program,
        transceiver_address,
    )
    .await
    .unwrap();

    (
        admin,
        integrator_program,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver_address,
        chain_id,
    )
}

async fn verify_transceiver_state(
    context: &mut ProgramTestContext,
    integrator_chain_config_pda: Pubkey,
    expected_recv_bitmap: u128,
    expected_send_bitmap: u128,
) {
    let integrator_chain_config: IntegratorChainConfig = get_account(&mut context.banks_client, integrator_chain_config_pda).await;

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
async fn test_set_in_transceivers_success() {
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

    let result = set_recv_transceiver(
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

    verify_transceiver_state(&mut context, integrator_chain_config_pda, 1, 0).await;
}

#[tokio::test]
async fn test_set_in_transceivers_multiple_sets_success() {
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

    // Set the first transceiver
    let result = set_recv_transceiver(
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

    // Register a second transceiver
    let transceiver2_address = Pubkey::new_unique();
    let (registered_transceiver2_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver2_address);

    register_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        registered_transceiver2_pda,
        integrator_program_id,
        transceiver2_address,
    )
    .await
    .unwrap();

    let result = set_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver2_pda,
        chain_id,
        transceiver2_address,
        integrator_program_id,
    )
    .await;
    assert!(result.is_ok());

    // Verify that both transceivers are set
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0b11, 0).await;
}

#[tokio::test]
async fn test_set_out_transceivers_success() {
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

    let result = set_send_transceiver(
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

    verify_transceiver_state(&mut context, integrator_chain_config_pda, 0, 1).await;
}

#[tokio::test]
async fn test_set_transceivers_invalid_authority() {
    let mut context = setup().await;
    let (
        _authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Create a new keypair to act as an invalid authority
    let invalid_authority = Keypair::new();
    let payer = context.payer.insecure_clone();

    let result = set_recv_transceiver(
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
            InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
        )
    );
}

#[tokio::test]
async fn test_set_transceivers_invalid_transceiver_id() {
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

    let result = set_recv_transceiver(
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
        TransactionError::InstructionError(0, InstructionError::Custom(2006))
    );
}

#[tokio::test]
async fn test_enable_already_enabled_transceiver() {
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

    // First attempt: should succeed
    let result = set_recv_transceiver(
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

    verify_transceiver_state(&mut context, integrator_chain_config_pda, 1, 0).await;

    // Second attempt: should fail with TransceiverAlreadyEnabled
    let result = set_recv_transceiver(
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

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::TransceiverAlreadyEnabled.into())
        )
    );

    // Verify that the state hasn't changed
    verify_transceiver_state(&mut context, integrator_chain_config_pda, 1, 0).await;
}
