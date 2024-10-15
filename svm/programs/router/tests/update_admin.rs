// #![cfg(feature = "test-sbf")]

// mod common;
// mod instructions;

// use crate::instructions::register::register;
// use crate::instructions::update_admin::update_admin;
// use anchor_lang::prelude::*;
// use common::setup::{get_account, setup};
// use router::error::RouterError;
// use router::state::IntegratorConfig;
// use solana_program_test::*;
// use solana_sdk::{
//     instruction::InstructionError, signature::Keypair, signer::Signer,
//     transaction::TransactionError,
// };

// async fn initialize_test_environment(
//     context: &mut ProgramTestContext,
// ) -> (Keypair, Keypair, Pubkey, Pubkey) {
//     let payer = context.payer.insecure_clone();
//     let owner = Keypair::new();
//     let integrator_program = Keypair::new();

//     let (integrator_config_pda, _) = Pubkey::find_program_address(
//         &[
//             IntegratorConfig::SEED_PREFIX,
//             integrator_program.pubkey().as_ref(),
//         ],
//         &router::id(),
//     );

//     // Initialize the integrator config
//     register(
//         context,
//         &payer,
//         owner.pubkey(),
//         integrator_config_pda,
//         &integrator_program,
//     )
//     .await
//     .unwrap();

//     (
//         owner,
//         payer,
//         integrator_program.pubkey(),
//         integrator_config_pda,
//     )
// }

// #[tokio::test]
// async fn test_update_admin_success() {
//     let mut context = setup().await;
//     let (current_owner, payer, integrator_program_id, integrator_config_pda) =
//         initialize_test_environment(&mut context).await;

//     let new_owner = Keypair::new();

//     update_admin(
//         &mut context,
//         &current_owner,
//         &new_owner,
//         &payer,
//         integrator_config_pda,
//         integrator_program_id,
//     )
//     .await
//     .unwrap();

//     // Verify that the ownership has been transferred
//     let integrator_config: IntegratorConfig =
//         get_account(&mut context.banks_client, integrator_config_pda).await;
//     assert_eq!(integrator_config.admin, new_owner.pubkey());
// }

// #[tokio::test]
// async fn test_update_admin_invalid_current_owner() {
//     let mut context = setup().await;
//     let (current_owner, payer, integrator_program_id, integrator_config_pda) =
//         initialize_test_environment(&mut context).await;

//     let invalid_owner = Keypair::new();
//     let new_owner = Keypair::new();

//     let result = update_admin(
//         &mut context,
//         &invalid_owner,
//         &new_owner,
//         &payer,
//         integrator_config_pda,
//         integrator_program_id,
//     )
//     .await;

//     assert_eq!(
//         result.unwrap_err().unwrap(),
//         TransactionError::InstructionError(
//             0,
//             InstructionError::Custom(RouterError::InvalidIntegratorAuthority.into())
//         )
//     );

//     // Verify that the ownership has not been transferred
//     let integrator_config: IntegratorConfig =
//         get_account(&mut context.banks_client, integrator_config_pda).await;
//     assert_eq!(integrator_config.admin, current_owner.pubkey());
// }

// #[tokio::test]
// async fn test_update_admin_same_owner() {
//     let mut context = setup().await;
//     let (current_owner, payer, integrator_program_id, integrator_config_pda) =
//         initialize_test_environment(&mut context).await;

//     let result = update_admin(
//         &mut context,
//         &current_owner,
//         &current_owner,
//         &payer,
//         integrator_config_pda,
//         integrator_program_id,
//     )
//     .await;

//     // The transaction should succeed, but the owner should remain the same
//     assert!(result.is_ok());

//     // Verify that the ownership has not changed
//     let integrator_config: IntegratorConfig =
//         get_account(&mut context.banks_client, integrator_config_pda).await;
//     assert_eq!(integrator_config.admin, current_owner.pubkey());
// }
