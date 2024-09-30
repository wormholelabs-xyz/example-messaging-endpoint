#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use instructions::{
    init_integrator_chain_transceivers::init_integrator_chain_transceivers,
    register_integrator::register_integrator, register_transceiver::register_transceiver,
};
use router::state::{Config, Integrator, IntegratorChainTransceivers, RegisteredTransceiver};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_register_transceiver_success() {
    // Set up the test environment
    let (mut context, owner, config_pda) = setup().await;
    let payer_pubkey = context.payer.pubkey();
    let integrator_authority = Keypair::new();

    // Register an integrator
    let config: Config = get_account(&mut context.banks_client, config_pda).await;
    let (integrator_pda, _) = Pubkey::find_program_address(
        &[
            Integrator::SEED_PREFIX,
            config.next_integrator_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    register_integrator(
        &mut context,
        &owner,
        config_pda,
        integrator_pda,
        integrator_authority.pubkey(),
    )
    .await
    .unwrap();

    // Initialize integrator chain transceivers
    let integrator: Integrator = get_account(&mut context.banks_client, integrator_pda).await;
    let chain_id: u16 = 1;
    let (integrator_chain_transceivers_pda, _) = Pubkey::find_program_address(
        &[
            IntegratorChainTransceivers::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    init_integrator_chain_transceivers(
        &mut context,
        config_pda,
        integrator_pda,
        integrator_chain_transceivers_pda,
        chain_id,
        payer_pubkey,
    )
    .await
    .unwrap();

    // Register a transceiver
    let (registered_transceiver_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
            0u64.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );
    let transceiver_address = Keypair::new().pubkey();

    register_transceiver(
        &mut context,
        config_pda,
        integrator_pda,
        &integrator_authority,
        registered_transceiver_pda,
        integrator_chain_transceivers_pda,
        chain_id,
        transceiver_address,
    )
    .await
    .unwrap();

    // Verify the IntegratorChainTransceivers account
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;
    assert_eq!(
        integrator_chain_transceivers.integrator_id,
        integrator.id as u64
    );
    assert_eq!(integrator_chain_transceivers.chain_id, chain_id);
    assert_eq!(integrator_chain_transceivers.next_transceiver_id, 1);
    assert_eq!(integrator_chain_transceivers.transceiver_bitmap, 1);

    // Verify the RegisteredTransceiver account
    let registered_transceiver: RegisteredTransceiver =
        get_account(&mut context.banks_client, registered_transceiver_pda).await;
    assert_eq!(registered_transceiver.integrator_id, integrator.id);
    assert_eq!(registered_transceiver.id, 0);
    assert_eq!(registered_transceiver.chain_id, chain_id);
    assert_eq!(registered_transceiver.address, transceiver_address);
}
