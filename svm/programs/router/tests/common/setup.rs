use anchor_lang::prelude::*;
use router::id;
use solana_program_test::{ProgramTest, ProgramTestContext};

pub struct TestContext {
    pub program_test_context: ProgramTestContext,
}

pub async fn setup() -> TestContext {
    // Set up the program test environment
    let program_id = id();
    let program_test = ProgramTest::new("router", program_id, None);

    // Start the test context
    let ctx = program_test.start_with_context().await;

    let test_context = TestContext {
        program_test_context: ctx,
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
