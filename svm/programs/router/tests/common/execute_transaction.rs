use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

pub async fn execute_transaction(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    signers: &[&Keypair],
    payer: &Keypair,
) -> Result<(), BanksClientError> {
    let recent_blockhash = context.get_new_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        signers,
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}
