use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::InitIntegratorChainTransceivers;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

use crate::common::setup::TestContext;

pub async fn init_integrator_chain_transceivers(
    context: &mut TestContext,
    config_pda: Pubkey,
    owner: &Keypair,
    payer: &Keypair,
    integrator_chain_transceivers: Pubkey,
    chain_id: u16,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = InitIntegratorChainTransceivers {
        config: config_pda,
        owner: owner.pubkey(),
        payer: payer.pubkey(),
        integrator_chain_transceivers,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::InitIntegratorChainTransceivers {
            chain_id,
            integrator_program_id,
        }
        .data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer, owner],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}
