#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers;
use crate::instructions::initialize_integrator_config::initialize_integrator_config;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::{IntegratorChainTransceivers, IntegratorConfig};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

async fn initialize_test_environment(
    context: &mut ProgramTestContext,
) -> (Keypair, Keypair, Pubkey, Pubkey, Pubkey, u16) {
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

    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program.pubkey().as_ref(),
            chain_id.to_le_bytes().as_ref(),
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

    (
        owner,
        payer,
        integrator_program.pubkey(),
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
    )
}
#[tokio::test]
async fn test_initialize_integrator_chain_transceivers_success() {
    let mut context = setup().await;
    let (_, _, integrator_program_id, _, integrator_chain_transceivers_pda, chain_id) =
        initialize_test_environment(&mut context).await;

    // Fetch and verify the initialized account
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;

    assert_eq!(integrator_chain_transceivers.chain_id, chain_id);
    assert_eq!(
        integrator_chain_transceivers.integrator_program_id,
        integrator_program_id
    );
    for i in 0..128 {
        assert!(integrator_chain_transceivers
            .send_transceiver_bitmap
            .is_empty());
        assert!(integrator_chain_transceivers
            .recv_transceiver_bitmap
            .is_empty());
    }
}

#[tokio::test]
async fn test_initialize_integrator_chain_transceivers_reinitialization() {
    let mut context = setup().await;
    let (
        authority,
        payer,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Try to initialize again
    let result = initialize_integrator_chain_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
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
async fn test_initialize_integrator_chain_transceivers_different_chains() {
    let mut context = setup().await;
    let (authority, payer, integrator_program_id, integrator_config_pda, _, _) =
        initialize_test_environment(&mut context).await;

    let chain_id_2: u16 = 2;
    let (integrator_chain_transceivers_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_2.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize for chain 2
    initialize_integrator_chain_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda_2,
        chain_id_2,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify both accounts
    let integrator_chain_transceivers_1: IntegratorChainTransceivers = get_account(
        &mut context.banks_client,
        integrator_chain_transceivers_pda_2,
    )
    .await;

    assert_eq!(integrator_chain_transceivers_1.chain_id, chain_id_2);
    assert_eq!(
        integrator_chain_transceivers_1.integrator_program_id,
        integrator_program_id
    );
}

#[tokio::test]
async fn test_initialize_integrator_chain_transceivers_invalid_authority() {
    let mut context = setup().await;
    let (_, payer, integrator_program_id, integrator_config_pda, _, chain_id) =
        initialize_test_environment(&mut context).await;

    // Create a different authority that wasn't used in the setup
    let different_authority = Keypair::new();

    let chain_id_2: u16 = 2;

    let (integrator_chain_transceivers_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_2.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Attempt to initialize with the different (invalid) authority
    let result = initialize_integrator_chain_transceivers(
        &mut context,
        &different_authority,
        &payer,
        integrator_config_pda,
        integrator_chain_transceivers_pda_2,
        chain_id_2,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to invalid transceiver id
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
        )
    );
}
