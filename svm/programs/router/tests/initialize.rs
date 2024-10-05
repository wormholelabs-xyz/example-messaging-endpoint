#![cfg(feature = "test-sbf")]

mod common;

use common::setup::{get_account, setup};
use router::state::Config;

#[tokio::test]
async fn test_initialize() {
    // Set up the test environment
    let (mut context, config_pda) = setup().await;

    // Verify the state after initialization
    let config: Config = get_account(&mut context.banks_client, config_pda).await;

    assert_eq!(config.next_integrator_id, 0);
    // Verify that the bump is set (it should be a non-zero value)
    assert_ne!(config.bump, 0);
}
