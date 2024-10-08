use anchor_lang::prelude::*;
use router::id;
use solana_program_test::ProgramTest;
use solana_sdk::{hash::Hash, signature::Keypair};

pub struct TestContext {
    pub banks_client: solana_program_test::BanksClient,
    pub payer: Keypair,
    pub last_blockhash: Hash,
}

pub async fn setup() -> TestContext {
    // Set up the program test environment
    let program_id = id();
    let program_test = ProgramTest::new("router", program_id, None);

    // Start the test context
    let ctx = program_test.start_with_context().await;

    let test_context = TestContext {
        banks_client: ctx.banks_client,
        payer: ctx.payer,
        last_blockhash: ctx.last_blockhash,
    };

    test_context
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
