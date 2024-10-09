#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::initialize_integrator_chain_transceivers::initialize_integrator_chain_transceivers;
use crate::instructions::initialize_integrator_config::initialize_integrator_config;
use crate::instructions::set_transceivers::set_transceivers;

use anchor_lang::prelude::*;
use common::setup::setup;
use router::{
    state::{IntegratorChainTransceivers, IntegratorConfig},
    utils::bitmap::Bitmap,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_set_in_transceivers_success() {
    // Set up the test environment
    let mut context = setup().await;
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

    // Initialize the integrator config
    initialize_integrator_config(
        &mut context,
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
        &mut context,
        &authority,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Set incoming transceivers
    let is_incoming = true;
    let bitmap: u128 = 0b1010101010101010;
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

    // Verify the state
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
        Bitmap::from_value(bitmap)
    );
    assert_eq!(
        integrator_chain_transceivers.out_transceiver_bitmap,
        Bitmap::new()
    );
}
