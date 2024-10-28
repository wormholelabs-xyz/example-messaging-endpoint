#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_transceiver::add_transceiver;
use crate::instructions::attest_message::attest_message;
use crate::instructions::enable_transceiver::enable_recv_transceiver;
use crate::instructions::exec_message::exec_message;
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

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Pubkey, Pubkey, u16) {
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
        chain_id,
    )
}

#[tokio::test]
async fn test_exec_message_success() {
    let (mut context, payer, _, _, integrator_chain_config_pda, chain_id) =
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

    let result = exec_message(
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

    assert!(result.is_ok(), "exec_message failed: {:?}", result.err());

    // Verify the attestation info account was created and initialized correctly
    let attestation_info: AttestationInfo =
        get_account(&mut context.banks_client, attestation_info_pda).await;
    assert_eq!(attestation_info.src_chain, src_chain);
    assert_eq!(attestation_info.src_addr, src_addr);
    assert_eq!(attestation_info.sequence, sequence);
    assert_eq!(attestation_info.dst_chain, dst_chain);
    assert_eq!(attestation_info.dst_addr, dst_addr);
    assert_eq!(attestation_info.payload_hash, payload_hash);
    assert!(attestation_info.executed);
}

#[tokio::test]
async fn test_exec_message_duplicate_execution() {
    let (mut context, payer, _, _, integrator_chain_config_pda, chain_id) =
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

    // First execution (should succeed)
    exec_message(
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

    // Second execution (should fail)
    let result = exec_message(
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
