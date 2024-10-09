use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::InitIntegratorConfig;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

pub async fn initialize_integrator_config(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    authority: Pubkey,
    integrator_config: Pubkey,
    integrator_program: &Keypair,
) -> Result<(), BanksClientError> {
    let accounts = InitIntegratorConfig {
        payer: payer.pubkey(),
        authority,
        integrator_config,
        integrator_program: integrator_program.pubkey(),
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::InitIntegratorConfig {}.data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer, integrator_program],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}
