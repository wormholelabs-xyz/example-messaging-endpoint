mod common;
mod instructions;

use anchor_lang::prelude::Pubkey;
use common::setup::{get_account, setup};
use instructions::register_integrator::register_integrator;
use instructions::register_transceiver::register_transceiver;
use router::state::{Config, Integrator, RegisteredTransceiver};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_register_transceiver_success() {
    // Set up the test environment
    let (mut context, owner, config_pda) = setup().await;

    // Create a new keypair for the integrator authority
    let integrator_authority = Keypair::new();

    // Get the current config
    let config: Config = get_account(&mut context.banks_client, config_pda).await;

    // Calculate the integrator PDA
    let (integrator_pda, _) = Pubkey::find_program_address(
        &[
            Integrator::SEED_PREFIX,
            config.next_integrator_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Register the integrator
    register_integrator(
        &mut context,
        &owner,
        config_pda,
        integrator_pda,
        integrator_authority.pubkey(),
    )
    .await
    .unwrap();

    // Get the updated integrator account
    let integrator: Integrator = get_account(&mut context.banks_client, integrator_pda).await;

    // Create a new keypair for the transceiver
    let transceiver_keypair = Keypair::new();

    // Calculate the registered transceiver PDA
    let (registered_transceiver_pda, _) = Pubkey::find_program_address(
        &[
            RegisteredTransceiver::SEED_PREFIX,
            integrator.id.to_le_bytes().as_ref(),
            integrator.next_transceiver_id.to_le_bytes().as_ref(),
        ],
        &router::id(),
    );

    // Register the transceiver
    register_transceiver(
        &mut context,
        config_pda,
        integrator_pda,
        &integrator_authority,
        registered_transceiver_pda,
        transceiver_keypair.pubkey(),
    )
    .await
    .unwrap();

    // Verify the RegisteredTransceiver account
    let registered_transceiver: RegisteredTransceiver =
        get_account(&mut context.banks_client, registered_transceiver_pda).await;
    assert_eq!(registered_transceiver.integrator_id, integrator.id);
    assert_eq!(registered_transceiver.id, 0);
    assert_eq!(registered_transceiver.address, transceiver_keypair.pubkey());

    // Verify the Integrator account
    let updated_integrator: Integrator =
        get_account(&mut context.banks_client, integrator_pda).await;
    assert_eq!(updated_integrator.next_transceiver_id, 1);
}
