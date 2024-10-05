use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::TransferIntegratorChainTransceiversOwnership;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

use crate::common::setup::TestContext;

pub async fn transfer_integrator_chain_transceivers_ownership(
    context: &mut TestContext,
    owner: &Keypair,
    integrator_chain_transceivers: Pubkey,
    new_owner: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = TransferIntegratorChainTransceiversOwnership {
        owner: owner.pubkey(),
        integrator_chain_transceivers,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::TransferIntegratorChainTransceiversOwnership { new_owner }
            .data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}
