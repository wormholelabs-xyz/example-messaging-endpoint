#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::disable_transceivers::{
    disable_recv_transceiver, disable_send_transceiver,
};
use crate::instructions::register::register;
use crate::instructions::register_transceiver::register_transceiver;
use crate::instructions::set_transceivers::{set_recv_transceiver, set_send_transceiver};

use anchor_lang::prelude::*;
use common::setup::setup;
use router::error::RouterError;
use router::{
    state::{IntegratorChainConfig, TransceiverInfo},
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

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            router::state::IntegratorConfig::SEED_PREFIX,
            mock_integrator::id().as_ref(),
        ],
        &router::id(),
    );

    register(
        context,
        &payer,
        &admin,
        integrator_config_pda,
        mock_integrator::id(),
    )
    .await
    .unwrap();

    // Initialize integrator chain transceivers
    let (integrator_chain_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainConfig::SEED_PREFIX,
            integrator_program.as_ref(),
            &chain_id.to_le_bytes(),
        ],
        &router::id(),
    );

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
    let account = context
        .banks_client
        .get_account(integrator_chain_config_pda)
        .await
        .unwrap()
        .unwrap();

    let integrator_chain_config: IntegratorChainConfig =
        IntegratorChainConfig::try_deserialize(&mut account.data.as_ref()).unwrap();

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
    set_recv_transceiver(
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
    set_send_transceiver(
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
async fn test_disable_transceivers_invalid_authority() {
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
    set_recv_transceiver(
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
            InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
        )
    );
}

#[tokio::test]
async fn test_disable_transceivers_invalid_transceiver_id() {
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
    set_recv_transceiver(
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


