#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_transceiver::add_transceiver;
use crate::instructions::enable_transceiver::enable_send_transceiver;
use crate::instructions::register::register;
use crate::instructions::send_message::send_message;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::{
    IntegratorChainConfig, IntegratorConfig, OutboxMessage, OutboxMessageKey, TransceiverInfo,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};
use universal_address::UniversalAddress;

async fn initialize_test_environment(
    context: &mut ProgramTestContext,
) -> (Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, u8, u16) {
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);
    let (integrator_program_pda, bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &integrator_program_id);

    // Register integrator
    register(
        context,
        &payer,
        &admin,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Setup chain config and transceiver
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);

    let transceiver_program_id = Keypair::new().pubkey();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);

    // Add and enable transceiver
    add_transceiver(
        context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await
    .unwrap();

    enable_send_transceiver(
        context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        chain_id,
        transceiver_program_id,
        integrator_program_id,
    )
    .await
    .unwrap();

    (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        integrator_program_pda,
        registered_transceiver_pda,
        bump,
        chain_id,
    )
}

#[tokio::test]
async fn test_send_message_success() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();

    let (
        _admin,
        integrator_program_id,
        _integrator_config_pda,
        integrator_chain_config_pda,
        integrator_program_pda,
        _registered_transceiver_pda,
        bump,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let dst_addr = UniversalAddress::from_bytes([1u8; 32]);
    let payload_hash = [2u8; 32];

    let (outbox_message_key_pda, _) = OutboxMessageKey::pda(&integrator_program_id);
    let outbox_message = Keypair::new();

    let result = send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        &outbox_message,
        outbox_message_key_pda,
        integrator_program_id,
        bump,
        chain_id,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_ok());

    // Verify outbox message was created correctly
    let outbox_msg: OutboxMessage =
        get_account(&mut context.banks_client, outbox_message.pubkey()).await;
    assert_eq!(
        outbox_msg.src_addr,
        UniversalAddress::from(integrator_program_pda)
    );
    assert_eq!(outbox_msg.sequence, 0);
    assert_eq!(outbox_msg.dst_chain, chain_id);
    assert_eq!(outbox_msg.dst_addr, dst_addr);
    assert_eq!(outbox_msg.payload_hash, payload_hash);
    assert_eq!(outbox_msg.outstanding_transceivers.as_value(), 1);
}

#[tokio::test]
async fn test_send_message_increments_sequence() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();

    let (
        _admin,
        integrator_program_id,
        _integrator_config_pda,
        integrator_chain_config_pda,
        integrator_program_pda,
        _registered_transceiver_pda,
        bump,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let dst_addr = UniversalAddress::from_bytes([1u8; 32]);
    let payload_hash = [2u8; 32];
    let (outbox_message_key_pda, _) = OutboxMessageKey::pda(&integrator_program_id);

    // Send first message
    let outbox_message_1 = Keypair::new();
    send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        &outbox_message_1,
        outbox_message_key_pda,
        integrator_program_id,
        bump,
        chain_id,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    // Verify first message sequence is 0
    let outbox_msg_1: OutboxMessage =
        get_account(&mut context.banks_client, outbox_message_1.pubkey()).await;
    assert_eq!(outbox_msg_1.sequence, 0);

    // Send second message
    let outbox_message_2 = Keypair::new();
    send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        &outbox_message_2,
        outbox_message_key_pda,
        integrator_program_id,
        bump,
        chain_id,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    // Verify second message sequence is 1
    let outbox_msg_2: OutboxMessage =
        get_account(&mut context.banks_client, outbox_message_2.pubkey()).await;
    assert_eq!(outbox_msg_2.sequence, 1);

    // Verify the sequence key was incremented
    let outbox_message_key: OutboxMessageKey =
        get_account(&mut context.banks_client, outbox_message_key_pda).await;
    assert_eq!(outbox_message_key.sequence, 2); // Next available sequence
}

#[tokio::test]
async fn test_send_message_no_enabled_transceivers() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();

    // Initialize without enabling any transceivers
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);
    let (integrator_program_pda, bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &integrator_program_id);
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);

    // Only register the integrator
    register(
        &mut context,
        &payer,
        &admin,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    let dst_addr = UniversalAddress::from_bytes([1u8; 32]);
    let payload_hash = [2u8; 32];
    let (outbox_message_key_pda, _) = OutboxMessageKey::pda(&integrator_program_id);
    let outbox_message = Keypair::new();

    let result = send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        integrator_chain_config_pda,
        &outbox_message,
        outbox_message_key_pda,
        integrator_program_id,
        bump,
        chain_id,
        dst_addr,
        payload_hash,
    )
    .await;

    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(3012)) // AccountNotInitialized
    );
}

#[tokio::test]
async fn test_send_message_unregistered_chain() {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();

    let (
        _admin,
        integrator_program_id,
        _integrator_config_pda,
        _integrator_chain_config_pda,
        integrator_program_pda,
        _registered_transceiver_pda,
        bump,
        _chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Try to send to an unregistered chain
    let unregistered_chain_id: u16 = 999;
    let (unregistered_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, unregistered_chain_id);

    let dst_addr = UniversalAddress::from_bytes([1u8; 32]);
    let payload_hash = [2u8; 32];
    let (outbox_message_key_pda, _) = OutboxMessageKey::pda(&integrator_program_id);
    let outbox_message = Keypair::new();

    let result = send_message(
        &mut context,
        &payer,
        integrator_program_pda,
        unregistered_chain_config_pda,
        &outbox_message,
        outbox_message_key_pda,
        integrator_program_id,
        bump,
        unregistered_chain_id,
        dst_addr,
        payload_hash,
    )
    .await;

    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(3012)) // AccountNotInitialized
    );
}