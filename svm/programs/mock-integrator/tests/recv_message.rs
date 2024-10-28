#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_transceiver::add_transceiver;
use crate::instructions::attest_message::attest_message;
use crate::instructions::enable_transceiver::enable_recv_transceiver;
use crate::instructions::recv_message::recv_message;
use crate::instructions::register::register;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use router::error::RouterError;
use router::state::{AttestationInfo, IntegratorChainConfig, IntegratorConfig, TransceiverInfo};
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
    u16,
) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 2;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&integrator_program_id);
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);

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

    // Setup transceiver
    let transceiver_program_id = mock_transceiver::id();
    let (registered_transceiver_pda, _) =
        TransceiverInfo::pda(&integrator_program_id, &transceiver_program_id);
    let (transceiver_pda, _) =
        Pubkey::find_program_address(&[b"transceiver_pda"], &transceiver_program_id);

    // Add and enable transceiver
    add_transceiver(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        registered_transceiver_pda,
        integrator_program_id,
        transceiver_program_id,
    )
    .await
    .unwrap();

    enable_recv_transceiver(
        &mut context,
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
        context,
        payer,
        admin,
        integrator_config_pda,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver_pda,
        chain_id,
    )
}

#[tokio::test]
async fn test_recv_message_success() {
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver_pda,
        chain_id,
    ) = setup_test_environment().await;

    let src_chain: u16 = chain_id;
    let src_addr = UniversalAddress::from_bytes([1u8; 32]);
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = UniversalAddress::from_pubkey(&mock_integrator::id());
    let payload_hash = [3u8; 32];

    let (attestation_info_pda, _) = AttestationInfo::pda(AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    ));

    // First, attest the message
    attest_message(
        &mut context,
        &payer,
        registered_transceiver_pda,
        transceiver_pda,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    // Now, receive the message
    let result = recv_message(
        &mut context,
        &payer,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_ok(), "recv_message failed: {:?}", result.err());

    // Verify the state after receiving the message
    let integrator_chain_config: IntegratorChainConfig =
        get_account(&mut context.banks_client, integrator_chain_config_pda).await;
    let attestation_info: AttestationInfo =
        get_account(&mut context.banks_client, attestation_info_pda).await;

    // Verify that the message is marked as executed
    assert!(attestation_info.executed);

    // Verify that the transceiver is still enabled
    assert!(integrator_chain_config
        .recv_transceiver_bitmap
        .get(0)
        .unwrap());

    // Verify that the transceiver has attested
    assert!(attestation_info.attested_transceivers.get(0).unwrap());

    // TODO: return data are assumed to be correct by checking the `recv_transceiver_bitmap`
    // and `attested_transceivers`. It is better to check the result of the transaction
	// It is not checked here yet as it requires some other set up to execute transaction
}

#[tokio::test]
async fn test_recv_message_already_executed() {
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        registered_transceiver_pda,
        transceiver_pda,
        chain_id,
    ) = setup_test_environment().await;

    let src_chain: u16 = chain_id;
    let src_addr = UniversalAddress::from_bytes([1u8; 32]);
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = UniversalAddress::from_pubkey(&mock_integrator::id());
    let payload_hash = [3u8; 32];

    let (attestation_info_pda, _) = AttestationInfo::pda(AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    ));

    // First, attest and receive the message
    attest_message(
        &mut context,
        &payer,
        registered_transceiver_pda,
        transceiver_pda,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    recv_message(
        &mut context,
        &payer,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    // Try to receive the message again
    let result = recv_message(
        &mut context,
        &payer,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(RouterError::AlreadyExecuted.into())
        )
    );
}

#[tokio::test]
async fn test_recv_message_no_attestation() {
    let (mut context, payer, _, _, integrator_chain_config_pda, _, _, chain_id) =
        setup_test_environment().await;

    let src_chain: u16 = chain_id;
    let src_addr = UniversalAddress::from_bytes([1u8; 32]);
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = UniversalAddress::from_pubkey(&mock_integrator::id());
    let payload_hash = [3u8; 32];

    let (attestation_info_pda, _) = AttestationInfo::pda(AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    ));

    // Try to receive a message that hasn't been attested
    let result = recv_message(
        &mut context,
        &payer,
        integrator_chain_config_pda,
        attestation_info_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_err());
    // The error should be related to the account not being initialized
    assert!(matches!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(_, InstructionError::Custom(3012))
    ));
}
