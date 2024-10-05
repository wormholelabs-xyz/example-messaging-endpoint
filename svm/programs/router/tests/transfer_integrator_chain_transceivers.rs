#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::{
    init_integrator_chain_transceivers::init_integrator_chain_transceivers,
    transfer_integrator_chain_transceivers_ownership::transfer_integrator_chain_transceivers_ownership,
};
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::state::IntegratorChainTransceivers;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_transfer_integrator_chain_transceivers_ownership() {
    // Set up the test environment
    let (mut context, config_pda) = setup().await;
    let payer = context.payer.insecure_clone();
    let initial_owner = Keypair::new();
    let new_owner = Keypair::new();
    let integrator_program_id = Keypair::new().pubkey();
    let chain_id: u16 = 1;

    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator_program_id.as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Initialize the integrator chain transceivers
    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        &initial_owner,
        &payer,
        integrator_chain_transceivers_pda,
        chain_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Transfer ownership

    transfer_integrator_chain_transceivers_ownership(
        &mut context,
        &initial_owner,
        integrator_chain_transceivers_pda,
        new_owner.pubkey(),
    )
    .await
    .unwrap();

    // Verify the ownership transfer
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;

    assert_eq!(integrator_chain_transceivers.owner, new_owner.pubkey());

    // Attempt to transfer ownership with the old owner (should fail)
    let result = transfer_integrator_chain_transceivers_ownership(
        &mut context,
        &initial_owner,
        integrator_chain_transceivers_pda,
        Keypair::new().pubkey(),
    )
    .await;

    assert!(result.is_err());
}
