use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::RegisterTransceiver;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

pub async fn register_transceiver(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    registered_transceiver: Pubkey,
    integrator_program: Pubkey,
    transceiver_address: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = RegisterTransceiver {
        payer: payer.pubkey(),
        authority: authority.pubkey(),
        integrator_config,
        registered_transceiver,
        integrator_program,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::RegisterTransceiver {
            transceiver_address,
        }
        .data(),
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
