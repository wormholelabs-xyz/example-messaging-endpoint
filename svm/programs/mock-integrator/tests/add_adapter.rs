#![cfg(feature = "test-sbf")]

mod common;
mod instructions;

use crate::instructions::add_adapter::add_adapter;
use crate::instructions::discard_admin::discard_admin;
use crate::instructions::register::register;
use crate::instructions::transfer_admin::transfer_admin;
use anchor_lang::prelude::*;
use common::setup::{get_account, setup};
use endpoint::error::EndpointError;
use endpoint::state::{AdapterInfo, IntegratorConfig};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    system_instruction::SystemError, transaction::TransactionError,
};

async fn setup_test_environment() -> (ProgramTestContext, Keypair, Keypair, Pubkey, Pubkey) {
    let mut context = setup().await;
    let payer = context.payer.insecure_clone();
    let admin = Keypair::new();

    let (integrator_config_pda, _) = IntegratorConfig::pda(&mock_integrator::id());

    register(
        &mut context,
        &payer,
        &admin,
        integrator_config_pda,
        mock_integrator::id(),
    )
    .await
    .unwrap();

    (
        context,
        payer,
        admin,
        mock_integrator::id(),
        integrator_config_pda,
    )
}

async fn register_test_adapter(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config_pda: Pubkey,
    integrator_program_id: Pubkey,
) -> (Pubkey, Pubkey) {
    let adapter_program_id = Keypair::new().pubkey();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);

    add_adapter(
        context,
        admin,
        payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await
    .unwrap();

    (adapter_program_id, adapter_info_pda)
}

#[tokio::test]
async fn test_add_adapter_success() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let (adapter_program_id, adapter_info_pda) = register_test_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Fetch and verify the registered adapter
    let adapter_info: AdapterInfo = get_account(&mut context.banks_client, adapter_info_pda).await;

    assert_eq!(adapter_info.index, 0);
    assert_eq!(adapter_info.integrator_program_id, integrator_program_id);
    assert_eq!(adapter_info.adapter_program_id, adapter_program_id);

    // Verify that the integrator config's adapters list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.adapter_infos.len(), 1);
    assert_eq!(integrator_config.adapter_infos[0], adapter_program_id);
}

#[tokio::test]
async fn test_register_multiple_adapters() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register two adapters
    let mut adapter_program_ides = Vec::new();
    for id in 0..2 {
        let (adapter_program_id, adapter_info_pda) = register_test_adapter(
            &mut context,
            &admin,
            &payer,
            integrator_config_pda,
            integrator_program_id,
        )
        .await;
        adapter_program_ides.push(adapter_program_id);

        // Fetch and verify the registered adapter
        let adapter_info: AdapterInfo =
            get_account(&mut context.banks_client, adapter_info_pda).await;

        assert_eq!(adapter_info.index, id as u8);
        assert_eq!(adapter_info.integrator_program_id, integrator_program_id);
        assert_eq!(adapter_info.adapter_program_id, adapter_program_id);
    }

    // Verify that the integrator config's adapters list has been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.adapter_infos.len(), 2);
    assert_eq!(integrator_config.adapter_infos, adapter_program_ides);
}

#[tokio::test]
async fn test_register_max_adapters() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register the maximum number of adapters
    for _ in 0..IntegratorConfig::MAX_ADAPTERS {
        register_test_adapter(
            &mut context,
            &admin,
            &payer,
            integrator_config_pda,
            integrator_program_id,
        )
        .await;
    }

    // Attempt to register one more adapter (should fail)
    let extra_adapter_program_id = Keypair::new().pubkey();
    let (extra_adapter_info_pda, _) =
        AdapterInfo::pda(&integrator_program_id, &extra_adapter_program_id);

    let result = add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        extra_adapter_info_pda,
        integrator_program_id,
        extra_adapter_program_id,
    )
    .await;

    // Verify that the transaction failed with the MaxAdaptersReached error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::MaxAdaptersReached.into())
        )
    );

    // Verify that the integrator config's adapters list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(
        integrator_config.adapter_infos.len(),
        IntegratorConfig::MAX_ADAPTERS
    );
}

#[tokio::test]
async fn test_add_adapter_reinitialization() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Register an adapter
    let (adapter_program_id, adapter_info_pda) = register_test_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        integrator_program_id,
    )
    .await;

    // Attempt to register the same adapter again
    let result = add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await;

    // Verify that the transaction failed with the appropriate error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(SystemError::AccountAlreadyInUse as u32)
        ),
    );

    // Verify that the integrator config's adapters list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.adapter_infos.len(), 1);
    assert_eq!(integrator_config.adapter_infos[0], adapter_program_id);
}

#[tokio::test]
async fn test_add_adapter_non_authority() {
    let (mut context, payer, _, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // Create a non-authority signer
    let non_authority = Keypair::new();

    // Attempt to register an adapter with non-authority signer
    let adapter_program_id = Keypair::new().pubkey();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);

    let result = add_adapter(
        &mut context,
        &non_authority, // Use non-authority signer
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await;

    // Verify that the transaction failed with the CallerNotAuthorized error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::CallerNotAuthorized.into())
        )
    );

    // Verify that the integrator config's adapters list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.adapter_infos.len(), 0);
}

#[tokio::test]
async fn test_add_adapter_with_transfer_in_progress() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    let pending_admin = Keypair::new();

    // First, initiate a transfer
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

    // Now try to add an adapter
    let adapter_program_id = Keypair::new().pubkey();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);

    let result = add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
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

    // Verify that the integrator config hasn't changed
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.admin, Some(admin.pubkey()));
    assert_eq!(
        integrator_config.pending_admin,
        Some(pending_admin.pubkey())
    );
    assert_eq!(integrator_config.adapter_infos.len(), 0);
}

#[tokio::test]
async fn test_add_adapter_with_immutable_config() {
    let (mut context, payer, admin, integrator_program_id, integrator_config_pda) =
        setup_test_environment().await;

    // First, discard the admin to make the config immutable
    discard_admin(&mut context, &admin, &payer, integrator_config_pda)
        .await
        .unwrap();

    // Now try to add an adapter
    let adapter_program_id = Keypair::new().pubkey();
    let (adapter_info_pda, _) = AdapterInfo::pda(&integrator_program_id, &adapter_program_id);

    let result = add_adapter(
        &mut context,
        &admin,
        &payer,
        integrator_config_pda,
        adapter_info_pda,
        integrator_program_id,
        adapter_program_id,
    )
    .await;

    // The transaction should fail due to immutable config
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(EndpointError::CallerNotAuthorized.into())
        )
    );

    // Verify that the integrator config's adapters list has not been updated
    let integrator_config: IntegratorConfig =
        get_account(&mut context.banks_client, integrator_config_pda).await;
    assert_eq!(integrator_config.adapter_infos.len(), 0);
}
