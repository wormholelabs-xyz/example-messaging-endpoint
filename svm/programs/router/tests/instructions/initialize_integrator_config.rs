use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::InitIntegratorConfig;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

use crate::common::setup::TestContext;

pub async fn initialize_integrator_config(
    context: &mut TestContext,
    config_pda: Pubkey,
    authority: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = InitIntegratorConfig {
        authority: authority.pubkey(),
        payer: payer.pubkey(),
        integrator_config,
        integrator_program,
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
        &[payer, authority],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}
