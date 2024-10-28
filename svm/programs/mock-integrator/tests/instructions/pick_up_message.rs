use anchor_lang::{InstructionData, ToAccountMetas};
use mock_transceiver::accounts::InvokePickUpMessage;
use solana_program_test::*;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::keypair::Keypair};

use crate::common::execute_transaction::execute_transaction;

pub async fn pick_up_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    outbox_message: Pubkey,
    transceiver_info: Pubkey,
    transceiver_pda: Pubkey,
    refund_recipient: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = InvokePickUpMessage {
        outbox_message,
        transceiver_info,
        transceiver_pda,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
        refund_recipient,
    };

    let ix = Instruction {
        program_id: mock_transceiver::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_transceiver::instruction::InvokePickUpMessage {}.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
