#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers;
use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use crate::instructions::register_transceiver::register_transceiver;
use crate::instructions::set_transceivers::{set_recv_transceiver, set_send_transceiver};

use anchor_lang::prelude::*;
use common::setup::setup;
use router::error::RouterError;
use router::{
    state::{IntegratorChainTransceivers, IntegratorConfig, RegisteredTransceiver},
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
    let owner = Keypair::new();
    let integrator_program = Keypair::new();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator config
    initialize_integrator_config(
        context,
        &payer,
        owner.pubkey(),
        integrator_config_pda,
        &integrator_program,
    )
    .await
    .unwrap();

    // Initialize integrator chain transceivers
    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
            &chain_id.to_le_bytes(),
        ],
        &router::id(),
    );

    initialize_integrator_chain_transceivers(
        context,
        &owner,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program.pubkey(),
    )
    .await
    .unwrap();

    // Register a transceiver
    let transceiver_address = Pubkey::new_unique(); // Generate a unique pubkey for the transceiver
    let (registered_transceiver_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
            transceiver_address.as_ref(),
        ],
        &router::id(),
    );

    register_transceiver(
        context,
        &owner,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program.pubkey(),
        transceiver_address,
    )
    .await
    .unwrap();

    (
        owner,
        integrator_program.pubkey(),
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        transceiver_address,
        chain_id,
    )
}

async fn verify_transceiver_state(
    context: &mut ProgramTestContext,
    integrator_chain_transceivers_pda: Pubkey,
    expected_recv_bitmap: u128,
    expected_send_bitmap: u128,
) {
    let account = context
        .banks_client
        .get_account(integrator_chain_transceivers_pda)
        .await
        .unwrap()
        .unwrap();

    let integrator_chain_transceivers: IntegratorChainTransceivers =
        IntegratorChainTransceivers::try_deserialize(&mut account.data.as_ref()).unwrap();

    assert_eq!(
        integrator_chain_transceivers.recv_transceiver_bitmap,
        Bitmap::from_value(expected_recv_bitmap)
    );
    assert_eq!(
        integrator_chain_transceivers.send_transceiver_bitmap,
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
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = true;
    let payer = context.payer.insecure_clone();

    let result = set_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    )
    .await;
    assert!(result.is_ok());

    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 1, 0).await;
}

#[tokio::test]
async fn test_set_in_transceivers_multiple_sets_success() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = true;
    let payer = context.payer.insecure_clone();

    // Set the first transceiver
    let result = set_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    )
    .await;
    assert!(result.is_ok());

    // Register a second transceiver
    let transceiver2_address = Pubkey::new_unique();
    let (registered_transceiver2_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator_program_id.as_ref(),
            transceiver2_address.as_ref(),
        ],
        &router::id(),
    );

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

    // Set the second transceiver
    let result = set_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver2_pda,
        transceiver2_address,
        chain_id,
    )
    .await;
    assert!(result.is_ok());

    // Verify that both transceivers are set
    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 0b11, 0).await;
}

#[tokio::test]
async fn test_set_out_transceivers_success() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = false;
    let payer = context.payer.insecure_clone();

    let result = set_send_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    )
    .await;

    assert!(result.is_ok());

    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 0, 1).await;
}

#[tokio::test]
async fn test_set_transceivers_invalid_authority() {
    let mut context = setup().await;
    let (
        _authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Create a new keypair to act as an invalid authority
    let invalid_authority = Keypair::new();
    let is_incoming = true;
    let payer = context.payer.insecure_clone();

    let result = set_recv_transceiver(
        &mut context,
        &invalid_authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver_pda,
        transceiver,
        chain_id,
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
    // Verify that the state hasn't changed
    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 0, 0).await;
}

#[tokio::test]
async fn test_set_transceivers_invalid_transceiver_id() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        registered_transceiver_pda,
        _transceiver,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = true;
    // Use an invalid transceiver pubkey
    let invalid_transceiver = Keypair::new().pubkey();
    let payer = context.payer.insecure_clone();

    let result = set_recv_transceiver(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        registered_transceiver_pda,
        invalid_transceiver,
        chain_id,
    )
    .await;

    // The transaction should fail due to invalid transceiver id
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(2006))
    );

    // Verify that the state hasn't changed
    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 0, 0).await;
}
