use anchor_lang::{
    prelude::*, solana_program::instruction::Instruction, system_program, InstructionData,
};
use router::{id, instructions::initialize::*, state::Config};
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};

pub struct Initialize {
    pub payer: Pubkey,
    pub config: Pubkey,
}

pub fn initialize(accounts: Initialize, args: InitializeArgs) -> Instruction {
    let data = router::instruction::Initialize { args };

    let accounts = router::accounts::Initialize {
        payer: accounts.payer,
        config: accounts.config,
        system_program: system_program::ID,
    };

    Instruction {
        program_id: id(),
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

#[tokio::test]
async fn test_initialize() {
    // Set up the program test environment
    let program_id = id();
    let program_test = ProgramTest::new("router", program_id, None);

    // Start the test context
    let mut ctx = program_test.start_with_context().await;

    // Generate a new keypair for the owner
    let owner = Keypair::new();

    // Derive the config PDA
    let (config_pda, _bump) = Pubkey::find_program_address(&[Config::SEED_PREFIX], &program_id);

    // Build the initialize instruction
    let initialize_accounts = Initialize {
        payer: ctx.payer.pubkey(),
        config: config_pda,
    };

    let args = InitializeArgs {
        owner: owner.pubkey(),
    };

    let ix = initialize(initialize_accounts, args);

    let recent_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        recent_blockhash,
    );

    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Verify the state after initialization
    let config_account = ctx
        .banks_client
        .get_account(config_pda)
        .await
        .unwrap()
        .unwrap();

    let config: Config = Config::try_deserialize(&mut config_account.data.as_ref()).unwrap();

    assert_eq!(config.owner, owner.pubkey());
    assert_eq!(config.paused, false);
    assert_eq!(config.next_integrator_id, 0);
}
