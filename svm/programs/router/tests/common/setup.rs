use anchor_lang::prelude::*;
use router::id;
use solana_program_test::{ProgramTest, ProgramTestContext};

pub async fn setup() -> ProgramTestContext {
    // Set up the program test environment
    let program_id = id();
    let program_test = ProgramTest::new("router", program_id, None);

    // Start the test context
    program_test.start_with_context().await
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
