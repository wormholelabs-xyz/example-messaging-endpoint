#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_adapter::add_adapter;
use crate::instructions::disable_adapter::disable_send_adapter;
use crate::instructions::enable_adapter::enable_send_adapter;
use crate::instructions::pick_up_message::pick_up_message;
use crate::instructions::register::register;
use crate::instructions::send_message::send_message;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use endpoint::error::EndpointError;
use endpoint::state::{
    AdapterInfo, IntegratorChainConfig, IntegratorConfig, OutboxMessage, SequenceTracker,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};
use universal_address::UniversalAddress;

async fn setup_test_environment() -> (
    ProgramTestContext,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    u8,
    u16,
) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);
    let (integrator_program_pda, bump) =
        Pubkey::find_program_address(&[b"endpoint_integrator"], &integrator_program_id);

    // Register integrator
    register(
        &mut context,
        &payer,
        &admin,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Setup adapter
    let adapter_program_id = mock_adapter::id();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);
    let (adapter_pda, _) = Pubkey::find_program_address(&[b"adapter_pda"], &adapter_program_id);

    // Add and enable adapter
    add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await
    .unwrap();

    enable_send_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter_program_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    (
        context,
        payer,
        admin,
        integrator_program_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        bump,
        chain_id,
    )
}

async fn create_and_send_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    integrator_program_pda: Pubkey,
    integrator_chain_config_pda: Pubkey,
    integrator_program_id: Pubkey,
    bump: u8,
    chain_id: u16,
) -> Keypair {
    let outbox_message = Keypair::new();
    let (sequence_tracker_pda, _) = SequenceTracker::pda(&integrator_program_id);
    let dst_addr = UniversalAddress::from_bytes([1u8; 32]);
    let payload_hash = [2u8; 32];

    send_message(
        context,
        payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        &outbox_message,
        sequence_tracker_pda,
        chain_id,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    outbox_message
}

#[tokio::test]
async fn test_pick_up_message_success() {
    let (
        mut context,
        payer,
        _,
        integrator_program_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        bump,
        chain_id,
    ) = setup_test_environment().await;

    let outbox_message = create_and_send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        mock_integrator::id(),
        bump,
        chain_id,
    )
    .await;

    // Pick up message
    let result = pick_up_message(
        &mut context,
        &payer,
        outbox_message.pubkey(),
        adapter_info_pda,
        adapter_pda,
        payer.pubkey(),
    )
    .await;

    assert!(result.is_ok(), "pick_up_message failed: {:?}", result.err());

    // Check if the outbox message account is closed
    let account = context
        .banks_client
        .get_account(outbox_message.pubkey())
        .await
        .expect("Failed to get account info");

    assert!(
        account.is_none(),
        "OutboxMessage account should be closed, but it still exists"
    );

    if account.is_none() {
        println!("OutboxMessage account closed as expected.");
    }
}

/// This test checks `disabled_adapter` attempting to pick up as well, since the `outstanding_tranceivers`
/// is copied directly from `enabled_adapters` at the point of `outbox_message` creation
#[tokio::test]
async fn test_pick_up_message_all_already_picked_up() {
    let (
        mut context,
        payer,
        _,
        integrator_program_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        bump,
        chain_id,
    ) = setup_test_environment().await;

    let outbox_message = create_and_send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        mock_integrator::id(),
        bump,
        chain_id,
    )
    .await;

    // Pick up the message once
    pick_up_message(
        &mut context,
        &payer,
        outbox_message.pubkey(),
        adapter_info_pda,
        adapter_pda,
        payer.pubkey(),
    )
    .await
    .unwrap();

    // Attempt to pick up the message again
    let result = pick_up_message(
        &mut context,
        &payer,
        outbox_message.pubkey(),
        adapter_info_pda,
        adapter_pda,
        payer.pubkey(),
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(3012)) // AccountNotInitialized
    );
}
