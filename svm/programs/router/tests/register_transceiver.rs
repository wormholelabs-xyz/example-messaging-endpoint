#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use instructions::{
    init_integrator_chain_transceivers::init_integrator_chain_transceivers,
    register_integrator::register_integrator, register_transceiver::register_transceiver,
};
use router::error::RouterError;
use router::instructions::TransceiverType;
use router::state::{Config, Integrator, IntegratorChainTransceivers, RegisteredTransceiver};
use router::utils::bitmap::Bitmap;
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

    // Register multiple transceivers (less than 128)
    let num_transceivers = 127;
    let mut expected_bitmap = Bitmap::new();

    for i in 0..num_transceivers {
        let (registered_transceiver_pda, _) = Pubkey::find_program_address(
            &[
                RegisteredTransceiver::SEED_PREFIX,
                integrator.id.to_le_bytes().as_ref(),
                chain_id.to_le_bytes().as_ref(),
                (i as u8).to_le_bytes().as_ref(),
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
            TransceiverType::In,
            transceiver_address,
        )
        .await
        .unwrap();

        expected_bitmap.set(i, true).unwrap();

        // Verify the RegisteredTransceiver account
        let registered_transceiver: RegisteredTransceiver =
            get_account(&mut context.banks_client, registered_transceiver_pda).await;
        assert_eq!(registered_transceiver.integrator_id, integrator.id);
        assert_eq!(registered_transceiver.id, i);
        assert_eq!(registered_transceiver.chain_id, chain_id);
        assert_eq!(registered_transceiver.address, transceiver_address);
    }

    // Verify the IntegratorChainTransceivers account
    let integrator_chain_transceivers: IntegratorChainTransceivers =
        get_account(&mut context.banks_client, integrator_chain_transceivers_pda).await;
    assert_eq!(
        integrator_chain_transceivers.integrator_id,
        integrator.id as u64
    );
    assert_eq!(integrator_chain_transceivers.chain_id, chain_id);
    assert_eq!(
        integrator_chain_transceivers.next_in_transceiver_id,
        num_transceivers
    );
    assert_eq!(
        integrator_chain_transceivers.in_transceiver_bitmap,
        expected_bitmap
    );
}

#[tokio::test]
async fn test_register_transceiver_bitmap_overflow() {
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

    // Register 128 transceivers
    for i in 0..128 {
        let (registered_transceiver_pda, _) = Pubkey::find_program_address(
            &[
                RegisteredTransceiver::SEED_PREFIX,
                integrator.id.to_le_bytes().as_ref(),
                chain_id.to_le_bytes().as_ref(),
                (i as u8).to_le_bytes().as_ref(),
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
            TransceiverType::In,
            transceiver_address,
        )
        .await
        .unwrap();
    }

    // Try to register the 129th transceiver
    let (registered_transceiver_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            chain_id.to_le_bytes().as_ref(),
            (128u8).to_le_bytes().as_ref(),
        ],
        &router::id(),
    );
    let transceiver_address = Keypair::new().pubkey();

    let result = register_transceiver(
        &mut context,
        config_pda,
        integrator_pda,
        &integrator_authority,
        registered_transceiver_pda,
        integrator_chain_transceivers_pda,
        chain_id,
        TransceiverType::In,
        transceiver_address,
    )
    .await;

    // Assert that the specific BitmapIndexOutOfBounds error is returned
    // 0x1773 is the hexadecimal representation of error code 6003 (BitmapIndexOutOfBounds)
    assert!(
        result.as_ref().unwrap_err().to_string().contains("0x1773"),
        "Expected error containing '0x1773', but got: {:?}",
        result
    );
}
