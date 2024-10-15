#![cfg(feature = "test-sbf")]

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use mock_integrator::program::MockIntegrator;
use router::program::Router;
use solana_program_test::*;
use solana_sdk::msg;
use solana_sdk::{instruction::Instruction, transaction::Transaction};
use solana_sdk::{signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, Keypair) {
    let mut program_test = ProgramTest::new("mock_integrator", mock_integrator::id(), None);

    // Add the router program to the test environment
    program_test.add_program("router", router::id(), None);

    let mut context = program_test.start_with_context().await;
    let payer = context.payer.insecure_clone();

    (context, payer)
}

#[tokio::test]
async fn test_invoke_register() {
    let (mut context, payer) = setup().await;

    // Create necessary keypairs
    let admin = Keypair::new();

    // Derive PDAs
    let (integrator_config, integrator_config_bump) = Pubkey::find_program_address(
        &[
            router::state::IntegratorConfig::SEED_PREFIX,
            mock_integrator::id().as_ref(),
        ],
        &router::id(),
    );

    let (integrator_program_pda, integrator_program_pda_bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &mock_integrator::id());

    // Use println! instead of msg!
    println!("Integrator Program PDA: {:?}", integrator_program_pda);
    println!(
        "Integrator Program PDA Bump: {:?}",
        integrator_program_pda_bump
    );

    // Build the invoke_register instruction
    let accounts = mock_integrator::accounts::InvokeRegister {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        integrator_program_pda,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
    };

    println!("Integrator Program ID: {:?}", mock_integrator::id());

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeRegister {
            args: router::instructions::RegisterArgs {
                integrator_program_id: mock_integrator::id(),
                integrator_config_bump,
                integrator_program_pda_bump,
            },
        }
        .data(),
    };

    // Create and sign the transaction
    let recent_blockhash = context.last_blockhash;
    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // Process the transaction
    let result = context.banks_client.process_transaction(transaction).await;

    // Assert that the transaction was successful
    assert!(
        result.is_ok(),
        "Failed to process transaction: {:?}",
        result
    );

    // Verify that the integrator config account was created and initialized correctly
    let integrator_config_account = context
        .banks_client
        .get_account(integrator_config)
        .await
        .expect("Failed to fetch integrator config account")
        .expect("Integrator config account not found");

    let integrator_config_data = router::state::IntegratorConfig::try_deserialize(
        &mut integrator_config_account.data.as_ref(),
    )
    .expect("Failed to deserialize integrator config");

    assert_eq!(integrator_config_data.admin, admin.pubkey());
    assert_eq!(
        integrator_config_data.integrator_program_id,
        mock_integrator::id()
    );
    assert!(integrator_config_data.registered_transceivers.is_empty());
}
