#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers;
use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use crate::instructions::set_transceivers::set_transceivers;

use anchor_lang::prelude::*;
use common::setup::setup;
use router::error::RouterError;
use router::{
    state::{IntegratorChainTransceivers, IntegratorConfig},
    utils::bitmap::Bitmap,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn initialize_test_environment(
    context: &mut ProgramTestContext,
) -> (Keypair, Pubkey, Pubkey, Pubkey, u16) {
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();

    // Initialize integrator config
    let (integrator_config_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    initialize_integrator_config(
        context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Initialize integrator chain transceivers
    let chain_id: u16 = 1;
    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            &chain_id.to_le_bytes(),
        ],
        &router::id(),
    );

    initialize_integrator_chain_transceivers(
        context,
        &authority,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
    )
}

async fn verify_transceiver_state(
    context: &mut ProgramTestContext,
    integrator_chain_transceivers_pda: Pubkey,
    expected_in_bitmap: u128,
    expected_out_bitmap: u128,
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
        integrator_chain_transceivers.in_transceiver_bitmap,
        Bitmap::from_value(expected_in_bitmap)
    );
    assert_eq!(
        integrator_chain_transceivers.out_transceiver_bitmap,
        Bitmap::from_value(expected_out_bitmap)
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
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = true;
    let bitmap: u128 = 0b1010101010101010;
    let payer = context.payer.insecure_clone();

    let result = set_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        chain_id,
        is_incoming,
        bitmap,
    )
    .await;
    assert!(result.is_ok());

    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, bitmap, 0).await;
}

#[tokio::test]
async fn test_set_out_transceivers_success() {
    let mut context = setup().await;
    let (
        authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let is_incoming = false;
    let bitmap: u128 = 0b1100110011001100;
    let payer = context.payer.insecure_clone();

    let result = set_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        chain_id,
        is_incoming,
        bitmap,
    )
    .await;

    assert!(result.is_ok());

    verify_transceiver_state(&mut context, integrator_chain_transceivers_pda, 0, bitmap).await;
}

#[tokio::test]
async fn test_set_transceivers_invalid_authority() {
    let mut context = setup().await;
    let (
        _authority,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Create a new keypair to act as an invalid authority
    let invalid_authority = Keypair::new();
    let is_incoming = true;
    let bitmap: u128 = 0b1010101010101010;
    let payer = context.payer.insecure_clone();

    let result = set_transceivers(
        &mut context,
        &invalid_authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        integrator_program_id,
        chain_id,
        is_incoming,
        bitmap,
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