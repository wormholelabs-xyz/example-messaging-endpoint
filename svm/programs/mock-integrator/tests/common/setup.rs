use anchor_lang::prelude::*;
use router::id as router_id;
use solana_program_test::{ProgramTest, ProgramTestContext};

pub async fn setup() -> ProgramTestContext {
    let mut program_test = ProgramTest::new("router", router_id(), None);
    program_test.add_program("mock_integrator", mock_integrator::id(), None);
    program_test.add_program("mock_transceiver", mock_transceiver::id(), None);

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
