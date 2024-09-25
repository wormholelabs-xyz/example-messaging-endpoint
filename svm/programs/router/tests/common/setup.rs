use anchor_lang::{
    prelude::*, solana_program::instruction::Instruction, system_program, InstructionData,
};
use router::{id, instructions::initialize::*, state::Config};
use solana_program_test::ProgramTest;
use solana_sdk::{
    hash::Hash,
    signature::{Keypair, Signer},
};

pub struct TestContext {
    pub banks_client: solana_program_test::BanksClient,
    pub payer: Keypair,
    pub last_blockhash: Hash,
}

pub struct Initialize {
    pub payer: Pubkey,
    pub config: Pubkey,
}

pub async fn setup() -> (TestContext, Pubkey, Pubkey) {
    // Set up the program test environment
    let program_id = id();
    let program_test = ProgramTest::new("router", program_id, None);

    // Start the test context
    let mut ctx = program_test.start_with_context().await;

    // Generate a new keypair for the owner
    let owner = Keypair::new();

    // Derive the config PDA
    let (config_pda, _bump) = Pubkey::find_program_address(&[Config::SEED_PREFIX], &program_id);

    // Initialize the program
    initialize_program(&mut ctx, &owner, config_pda).await;

    let test_context = TestContext {
        banks_client: ctx.banks_client,
        payer: ctx.payer,
        last_blockhash: ctx.last_blockhash,
    };

    (test_context, owner.pubkey(), config_pda)
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

async fn initialize_program(
    ctx: &mut solana_program_test::ProgramTestContext,
    owner: &Keypair,
    config_pda: Pubkey,
) {
    let initialize_accounts = Initialize {
        payer: ctx.payer.pubkey(),
        config: config_pda,
    };

    let args = InitializeArgs {
        owner: owner.pubkey(),
    };

    let ix = initialize(initialize_accounts, args);

    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );

    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

pub async fn get_account<T: AccountDeserialize>(
    banks_client: &mut solana_program_test::BanksClient,
    address: Pubkey,
) -> T {
    let account = banks_client
        .get_account(address)
        .await
        .unwrap()
        .expect("account not found");

    T::try_deserialize(&mut account.data.as_ref()).unwrap()
}
