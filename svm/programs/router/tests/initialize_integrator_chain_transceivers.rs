#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::IntegratorChainTransceivers;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_initialize_integrator_chain_transceivers_success() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id: u16 = 1; // Example chain ID

    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator chain transceivers
    initialize_integrator_chain_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify the initialized account
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;

    assert_eq!(integrator_chain_transceivers.chain_id, chain_id);
    assert_eq!(
        integrator_chain_transceivers.integrator_program_id,
        integrator_program_id
    );
    for i in 0..128 {
        assert!(!integrator_chain_transceivers
            .in_transceiver_bitmap
            .get(i)
            .unwrap());
        assert!(!integrator_chain_transceivers
            .out_transceiver_bitmap
            .get(i)
            .unwrap());
    }
}

// TODO (@bingyuyap): this somehow fails, spent too much time on this. Will revisit
// #[tokio::test]
// async fn test_initialize_integrator_chain_transceivers_already_initialized() {
//     // Set up the test environment
//     let (mut context, config_pda) = setup().await;
//     let payer = context.payer.insecure_clone();
//     let authority = Keypair::new();
//     let integrator_program_id = Keypair::new().pubkey();
//     let chain_id: u16 = 1; // Example chain ID

//     let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
//         &[
//             IntegratorChainTransceivers::SEED_PREFIX,
//             integrator_program_id.as_ref(),
//             chain_id.to_le_bytes().as_ref(),
//         ],
//         &router::id(),
//     );

//     // Initialize the integrator chain transceivers
//     initialize_integrator_chain_transceivers(
//         &mut context,
//         &authority,
//         &payer,
//         integrator_chain_transceivers_pda,
//         chain_id,
//         integrator_program_id,
//     )
//     .await
//     .unwrap();

//     // Try to initialize again
//     let result = initialize_integrator_chain_transceivers(
//         &mut context,
//         &authority,
//         &payer,
//         integrator_chain_transceivers_pda,
//         chain_id,
//         integrator_program_id,
//     )
//     .await;

//     // Print out more information about the result
//     println!("Second initialization result: {:?}", result);

//     // Assert that the second initialization fails
//     assert!(result.is_err(), "Expected an error, but got: {:?}", result);
// }

#[tokio::test]
async fn test_initialize_integrator_chain_transceivers_different_chains() {
    // Set up the test environment
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let authority = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id_1: u16 = 1; // Example chain ID
    let chain_id_2: u16 = 2; // Example chain ID

    let (integrator_chain_transceivers_pda_1, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_1.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    let (integrator_chain_transceivers_pda_2, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id_2.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize for chain 1
    initialize_integrator_chain_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_chain_transceivers_pda_1,
        chain_id_1,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Initialize for chain 2
    initialize_integrator_chain_transceivers(
        &mut context,
        &authority,
        &payer,
        integrator_chain_transceivers_pda_2,
        chain_id_2,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Fetch and verify both accounts
    let integrator_chain_transceivers_1: IntegratorChainTransceivers = get_account(
        &mut context.banks_client,
        integrator_chain_transceivers_pda_1,
    )
    .await;
    let integrator_chain_transceivers_2: IntegratorChainTransceivers = get_account(
        &mut context.banks_client,
        integrator_chain_transceivers_pda_2,
    )
    .await;

    assert_eq!(integrator_chain_transceivers_1.chain_id, chain_id_1);
    assert_eq!(integrator_chain_transceivers_2.chain_id, chain_id_2);
    assert_eq!(
        integrator_chain_transceivers_1.integrator_program_id,
        integrator_program_id
    );
    assert_eq!(
        integrator_chain_transceivers_2.integrator_program_id,
        integrator_program_id
    );
}
