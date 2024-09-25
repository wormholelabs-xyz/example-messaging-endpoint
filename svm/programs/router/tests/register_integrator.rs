mod common;
mod instructions;

use anchor_lang::prelude::Pubkey;
use common::setup::{get_account, setup};
use instructions::register_integrator::register_integrator;
use router::state::{Config, Integrator};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_register_integrator_success() {
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

    // Verify the Integrator account
    let integrator: Integrator = get_account(&mut context.banks_client, integrator_pda).await;
    assert_eq!(integrator.id, 0);
    assert_eq!(integrator.authority, integrator_authority.pubkey());
    assert_eq!(integrator.next_transceiver_id, 0);

    // Verify the Config account
    let config: Config = get_account(&mut context.banks_client, config_pda).await;
    assert_eq!(config.next_integrator_id, 1);
}
