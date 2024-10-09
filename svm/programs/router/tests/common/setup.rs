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
