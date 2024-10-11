use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::SetTransceiver;
use router::instructions::SetTransceiverArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn set_transceivers(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    chain_id: u16,
    is_incoming: bool,
) -> Result<(), BanksClientError> {
    let accounts = SetTransceiver {
        payer: payer.pubkey(),
        owner: owner.pubkey(),
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
    };

    let args = SetTransceiverArgs { chain_id };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: if is_incoming {
            router::instruction::SetInTransceiver { args }.data()
        } else {
            router::instruction::SetOutTransceiver { args }.data()
        },
    };
    execute_transaction(context, ix, &[owner, payer], payer).await
}
