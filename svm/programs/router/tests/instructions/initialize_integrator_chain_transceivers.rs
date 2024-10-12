use anchor_lang::{InstructionData, ToAccountMetas};
use router::{
    accounts::InitializeIntegratorChainTransceivers,
    instructions::InitializeIntegratorChainTransceiversArgs,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn initialize_integrator_chain_transceivers(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    chain_id: u16,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = InitializeIntegratorChainTransceivers {
        owner: owner.pubkey(),
        payer: payer.pubkey(),
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::InitializeIntegratorChainTransceivers {
            args: InitializeIntegratorChainTransceiversArgs { chain_id },
        }
        .data(),
    };

    execute_transaction(context, ix, &[owner, payer], payer).await
}
