// use anchor_lang::{InstructionData, ToAccountMetas};
// use router::accounts::Register;
// use router::instructions::RegisterArgs;
// use solana_program_test::*;
// use solana_sdk::{
//     instruction::Instruction,
//     pubkey::Pubkey,
//     signer::{keypair::Keypair, Signer},
// };

// use crate::common::execute_transaction::execute_transaction;

// pub async fn register(
//     context: &mut ProgramTestContext,
//     payer: &Keypair,
//     admin: Pubkey,
//     integrator_config: Pubkey,
//     integrator_program_id: Pubkey,
// ) -> Result<(), BanksClientError> {
//     let (integrator_program_pda, integrator_program_pda_bump) = Pubkey::find_program_address(
//         &[b"router_integrator", integrator_program_id.as_ref()],
//         &integrator_program_id,
//     );

//     let accounts = Register {
//         payer: payer.pubkey(),
//         admin,
//         integrator_config,
//         integrator_program_pda,
//         system_program: solana_sdk::system_program::id(),
//     };

//     let args = RegisterArgs {
//         integrator_program_id,
//         0,
//         integrator_program_pda_bump,
//     };

//     let ix = Instruction {
//         program_id: router::id(),
//         accounts: accounts.to_account_metas(None),
//         data: router::instruction::Register { args }.data(),
//     };

//     execute_transaction(context, ix, &[payer], payer).await
// }
