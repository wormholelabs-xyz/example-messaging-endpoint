#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_adapter::add_adapter;
use crate::instructions::attest_message::attest_message;
use crate::instructions::enable_adapter::enable_recv_adapter;
use crate::instructions::exec_message::exec_message;
use crate::instructions::register::register;

use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use endpoint::error::EndpointError;
use endpoint::state::{AdapterInfo, AttestationInfo, IntegratorChainConfig, IntegratorConfig};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, transaction::TransactionError,
};

async fn setup_test_environment(
    chain_id: u16,
) -> (
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

    enable_recv_adapter(
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
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        chain_id,
    )
}

#[tokio::test]
async fn test_attest_message_success() {
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        chain_id,
    ) = setup_test_environment(2).await;

    let src_chain: u16 = chain_id;
    let src_addr = [1u8; 32];
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = mock_integrator::id().to_bytes();
    let payload_hash = [3u8; 32];

    let result = attest_message(
        &mut context,
        &payer,
        adapter_info_pda,
        adapter_pda,
        integrator_chain_config_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_ok(), "attest_message failed: {:?}", result.err());

    // Verify the attestation info account was created and initialized correctly
    let (attestation_info_pda, _) = AttestationInfo::pda(AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    ));
    let attestation_info: AttestationInfo =
        get_account(&mut context.banks_client, attestation_info_pda).await;
    assert_eq!(attestation_info.src_chain, src_chain);
    assert_eq!(attestation_info.src_addr, src_addr);
    assert_eq!(attestation_info.sequence, sequence);
    assert_eq!(attestation_info.dst_chain, dst_chain);
    assert_eq!(attestation_info.dst_addr, dst_addr);
    assert_eq!(attestation_info.payload_hash, payload_hash);
    assert_eq!(attestation_info.num_attested, 1);

    // Verify that the adapter's bit is set in the attested_adapters bitmap
    let adapter_info: AdapterInfo = get_account(&mut context.banks_client, adapter_info_pda).await;
    assert!(attestation_info
        .attested_adapters
        .get(adapter_info.index)
        .unwrap());
}

#[tokio::test]
async fn test_attest_message_after_exec() {
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        chain_id,
    ) = setup_test_environment(2).await;

    let src_chain: u16 = chain_id;
    let src_addr = [1u8; 32];
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = mock_integrator::id().to_bytes();
    let payload_hash = [3u8; 32];

    // First execution (should succeed)
    exec_message(
        &mut context,
        &payer,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    let result = attest_message(
        &mut context,
        &payer,
        adapter_info_pda,
        adapter_pda,
        integrator_chain_config_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await;

    assert!(result.is_ok(), "attest_message failed: {:?}", result.err());

    // Verify the attestation info account was created and initialized correctly
    let (attestation_info_pda, _) = AttestationInfo::pda(AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    ));
    let attestation_info: AttestationInfo =
        get_account(&mut context.banks_client, attestation_info_pda).await;
    assert_eq!(attestation_info.src_chain, src_chain);
    assert_eq!(attestation_info.src_addr, src_addr);
    assert_eq!(attestation_info.sequence, sequence);
    assert_eq!(attestation_info.dst_chain, dst_chain);
    assert_eq!(attestation_info.dst_addr, dst_addr);
    assert_eq!(attestation_info.payload_hash, payload_hash);

    // Verify that the adapter's bit is set in the attested_adapters bitmap
    let adapter_info: AdapterInfo = get_account(&mut context.banks_client, adapter_info_pda).await;
    assert!(attestation_info
        .attested_adapters
        .get(adapter_info.index)
        .unwrap());
}

#[tokio::test]
async fn test_attest_message_duplicate_attestation() {
    // Setup similar to the success test
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        chain_id,
    ) = setup_test_environment(2).await;

    let src_chain: u16 = chain_id;
    let src_addr = [1u8; 32];
    let sequence: u64 = 1;
    let dst_chain = 1;
    let dst_addr = mock_integrator::id().to_bytes();
    let payload_hash = [3u8; 32];

    // First attestation (should succeed)
    attest_message(
        &mut context,
        &payer,
        adapter_info_pda,
        adapter_pda,
        integrator_chain_config_pda,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    )
    .await
    .unwrap();

    // Second attestation (should fail)
    let result = attest_message(
        &mut context,
        &payer,
        adapter_info_pda,
        adapter_pda,
        integrator_chain_config_pda,
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
            InstructionError::Custom(EndpointError::DuplicateMessageAttestation.into())
        )
    );
}

#[tokio::test]
async fn test_attest_message_invalid_destination_chain() {
    let (
        mut context,
        payer,
        _,
        _,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_pda,
        chain_id,
    ) = setup_test_environment(2).await;

    let src_chain: u16 = chain_id;
    let src_addr = [1u8; 32];
    let sequence: u64 = 1;
    let dst_chain = 3; // Invalid destination chain
    let dst_addr = mock_integrator::id().to_bytes();
    let payload_hash = [3u8; 32];

    let result = attest_message(
        &mut context,
        &payer,
        adapter_info_pda,
        adapter_pda,
        integrator_chain_config_pda,
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
            InstructionError::Custom(EndpointError::InvalidDestinationChain.into())
        )
    );
}

// TODO: test using disabled_adapter. Need to find out how to make two adapters without having to duplicate the program
