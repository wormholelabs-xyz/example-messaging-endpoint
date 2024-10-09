use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::SetTransceivers;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

pub async fn set_transceivers(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    chain_id: u16,
    is_incoming: bool,
    bitmap: u128,
) -> Result<(), BanksClientError> {
    let accounts = SetTransceivers {
        payer: payer.pubkey(),
        authority: authority.pubkey(),
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: if is_incoming {
            router::instruction::SetInTransceivers { chain_id, bitmap }.data()
        } else {
            router::instruction::SetOutTransceivers { chain_id, bitmap }.data()
        },
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
