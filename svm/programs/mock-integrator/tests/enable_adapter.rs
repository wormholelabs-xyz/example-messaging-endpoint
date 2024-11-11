#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_adapter::add_adapter;
use crate::instructions::discard_admin::discard_admin;
use crate::instructions::enable_adapter::{enable_recv_adapter, enable_send_adapter};
use crate::instructions::register::register;

use crate::instructions::transfer_admin::transfer_admin;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use endpoint::error::EndpointError;
use endpoint::{
    state::{AdapterInfo, IntegratorChainConfig, IntegratorConfig},
    utils::bitmap::Bitmap,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn initialize_test_environment(
    context: &mut ProgramTestContext,
) -> (Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, u16) {
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();
    let integrator_program_id = mock_integrator::id();
    let chain_id: u16 = 1;

    let (integrator_config_pda, _) = IntegratorConfig::pda(&mock_integrator::id());

    register(
        context,
        &payer,
        &admin,
        integrator_config_pda,
        mock_integrator::id(),
    )
    .await
    .unwrap();

    // Prepare integrator_chain_config_pda
    let (integrator_chain_config_pda, _) =
        IntegratorChainConfig::pda(&integrator_program_id, chain_id);

    // Register an adapter
    let adapter_program_id = Keypair::new().pubkey();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);

    add_adapter(
        context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await
    .unwrap();

    (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter_program_id,
        chain_id,
    )
}

async fn verify_adapter_state(
    context: &mut ProgramTestContext,
    integrator_chain_config_pda: Pubkey,
    expected_recv_bitmap: u128,
    expected_send_bitmap: u128,
    expected_chain_id: u16,
    expected_integrator_id: Pubkey,
) {
    let integrator_chain_config: IntegratorChainConfig =
        get_account(&mut context.banks_client, integrator_chain_config_pda).await;

    assert_eq!(
        integrator_chain_config.recv_adapter_bitmap,
        Bitmap::from_value(expected_recv_bitmap)
    );
    assert_eq!(
        integrator_chain_config.send_adapter_bitmap,
        Bitmap::from_value(expected_send_bitmap)
    );
    assert_eq!(integrator_chain_config.chain_id, expected_chain_id);
    assert_eq!(
        integrator_chain_config.integrator_program_id,
        expected_integrator_id
    );
}

#[tokio::test]
async fn test_enable_in_adapters_success() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;
    assert!(result.is_ok());

    verify_adapter_state(
        &mut context,
        integrator_chain_config_pda,
        1,
        0,
        chain_id,
        integrator_program_id,
    )
    .await;
}

#[tokio::test]
async fn test_enable_in_adapters_multiple_sets_success() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Set the first adapter
    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;
    assert!(result.is_ok());

    // Register a second adapter
    let adapter2_address = Pubkey::new_unique();
    let (adapter_info2_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter2_address);

    add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info2_pda,
        integrator_program_id,
        adapter2_address,
    )
    .await
    .unwrap();

    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info2_pda,
        chain_id,
        adapter2_address,
        integrator_program_id,
    )
    .await;
    assert!(result.is_ok());

    // Verify that both adapters are set

    verify_adapter_state(
        &mut context,
        integrator_chain_config_pda,
        3,
        0,
        chain_id,
        integrator_program_id,
    )
    .await;
}

#[tokio::test]
async fn test_enable_out_adapters_success() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    let result = enable_send_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    assert!(result.is_ok());

    verify_adapter_state(
        &mut context,
        integrator_chain_config_pda,
        0,
        1,
        chain_id,
        integrator_program_id,
    )
    .await;
}

#[tokio::test]
async fn test_enable_adapter_invalid_admin() {
    let mut context = setup().await;
    let (
        _admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Create a new keypair to act as an invalid admin
    let invalid_admin = Keypair::new();
    let payer = context.payer.insecure_clone();

    let result = enable_recv_adapter(
        &mut context,
        &invalid_admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to invalid admin
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::CallerNotAuthorized.into())
        )
    );
}

#[tokio::test]
async fn test_enable_adapter_invalid_adapter_id() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        _adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    // Use an invalid adapter pubkey
    let invalid_adapter = Keypair::new().pubkey();
    let payer = context.payer.insecure_clone();

    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        invalid_adapter,
        integrator_program_id,
    )
    .await;

    // The transaction should fail due to invalid adapter id
    // It will return AccountNotInitialized because the adapter is not registered
    let err = result.unwrap_err();

    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(2006))
    );
}

#[tokio::test]
async fn test_enable_already_enabled_adapter() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // First attempt: should succeed
    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;
    assert!(result.is_ok());

    verify_adapter_state(
        &mut context,
        integrator_chain_config_pda,
        1,
        0,
        chain_id,
        integrator_program_id,
    )
    .await;

    // Second attempt: should fail with AdapterAlreadyEnabled
    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::AdapterAlreadyEnabled.into())
        )
    );

    // Verify that the state hasn't changed
    verify_adapter_state(
        &mut context,
        integrator_chain_config_pda,
        1,
        0,
        chain_id,
        integrator_program_id,
    )
    .await;
}

#[tokio::test]
async fn test_enable_adapter_with_transfer_in_progress() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();
    let pending_admin = Keypair::new();

    // Initiate an admin transfer
    transfer_admin(
        &mut context,
        &admin,
        &pending_admin.pubkey(),
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await
    .unwrap();

    // Now try to enable the receive adapter
    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::AdminTransferInProgress.into())
        )
    );

    // Verify that the IntegratorChainConfig account doesn't exist
    let chain_config_account = context
        .banks_client
        .get_account(integrator_chain_config_pda)
        .await
        .expect("Failed to get account");
    assert!(
        chain_config_account.is_none(),
        "IntegratorChainConfig account should not exist"
    );

    // Try to enable the send adapter
    let result = enable_send_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::AdminTransferInProgress.into())
        )
    );

    // Verify that the IntegratorChainConfig account still doesn't exist
    let chain_config_account = context
        .banks_client
        .get_account(integrator_chain_config_pda)
        .await
        .expect("Failed to get account");
    assert!(
        chain_config_account.is_none(),
        "IntegratorChainConfig account should not exist"
    );
}

#[tokio::test]
async fn test_enable_adapter_with_immutable_config() {
    let mut context = setup().await;
    let (
        admin,
        integrator_program_id,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        adapter,
        chain_id,
    ) = initialize_test_environment(&mut context).await;

    let payer = context.payer.insecure_clone();

    // Discard the admin to make the config immutable
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    // Now try to enable the receive adapter
    let result = enable_recv_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_chain_config_pda,
        adapter_info_pda,
        chain_id,
        adapter,
        integrator_program_id,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::CallerNotAuthorized.into())
        )
    );

    // Verify that the integrator config is immutable
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, None);
}
