// mod setup;
//
// use crate::setup::{setup_program, BSOL_MINT, PROGRAM_ID, USDC_MINT, WHIRLPOOL_ADDRESS};
// use borsh::BorshDeserialize;
// use paycheck::consts::PAYCHECK_SEED;
// use paycheck::instructions::{execute_paycheck_ix, CreatePaycheckArgs};
// use paycheck::state::Paycheck;
// use paycheck::ID;
// use solana_program::instruction::{AccountMeta, Instruction};
// use solana_program::program_option::COption;
// use solana_program::program_pack::Pack;
// use solana_program::pubkey::Pubkey;
// use solana_program_test::tokio;
// use solana_sdk::account::Account;
// use solana_sdk::pubkey;
// use solana_sdk::signature::{Keypair, SeedDerivable, Signer};
// use solana_sdk::transaction::Transaction;
// use spl_token::state::AccountState;
// use whirlpools_state::{SwapArgs, TOKEN_PROGRAM_ID};
//
//
// const SWAP_DISCRIMINATOR: [u8; 8] = [0x2b, 0x04, 0xed, 0x0b, 0x1a, 0xc9, 0x1e, 0x62];
//
// #[tokio::test]
// async fn try_swap() {
//     let program_id = PROGRAM_ID;
//     let (mut banks_client, payer, recent_blockhash, owner, token_account_b) =
//         setup_program(|p| {}).await;
//     let token_account_address =
//         spl_associated_token_account::get_associated_token_address(&owner.pubkey(), &BSOL_MINT);
//     let oracle = Pubkey::find_program_address(
//         &[b"oracle", WHIRLPOOL_ADDRESS.as_ref()],
//         &whirlpools_state::ID,
//     )
//     .0;
//     let whirlpool = whirlpools_state::Whirlpool::try_from_slice(
//         &banks_client
//             .get_account(WHIRLPOOL_ADDRESS)
//             .await
//             .unwrap()
//             .unwrap()
//             .data,
//     )
//     .unwrap();
//
//     let index_spacing = (whirlpool.tick_spacing as i32) * 88;
//     let start_tick_index =
//         whirlpool.tick_current_index - (whirlpool.tick_current_index % index_spacing);
//     let tick_array_0 = Pubkey::find_program_address(
//         &[
//             b"tick_array",
//             WHIRLPOOL_ADDRESS.as_ref(),
//             start_tick_index.to_string().as_bytes(),
//         ],
//         &whirlpools_state::ID,
//     )
//     .0;
//     let tick_array_1 = Pubkey::find_program_address(
//         &[
//             b"tick_array",
//             WHIRLPOOL_ADDRESS.as_ref(),
//             (start_tick_index - index_spacing).to_string().as_bytes(),
//         ],
//         &whirlpools_state::ID,
//     )
//     .0;
//     let tick_array_2 = Pubkey::find_program_address(
//         &[
//             b"tick_array",
//             WHIRLPOOL_ADDRESS.as_ref(),
//             (start_tick_index - index_spacing * 2)
//                 .to_string()
//                 .as_bytes(),
//         ],
//         &whirlpools_state::ID,
//     )
//     .0;
//
//     let input_args = SwapArgs {
//         swap_discriminator: SWAP_DISCRIMINATOR,
//         amount: 1000,
//         other_amount_threshold: u64::MAX,
//         sqrt_price_limit: 0,
//         amount_specified_is_input: false,
//         a_to_b: true,
//         remaining_accounts_info: None,
//     };
//
//     let swap_ix = Instruction::new_with_borsh(
//         whirlpools_state::ID,
//         &input_args,
//         vec![
//             AccountMeta::new_readonly(spl_token::id(), false),
//             AccountMeta::new_readonly(spl_token::id(), false),
//             AccountMeta::new_readonly(
//                 pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
//                 false,
//             ),
//             AccountMeta::new(owner.pubkey(), true),
//             AccountMeta::new(WHIRLPOOL_ADDRESS, false),
//             AccountMeta::new(BSOL_MINT, false),
//             AccountMeta::new(USDC_MINT, false),
//             AccountMeta::new(token_account_address, false),
//             AccountMeta::new(whirlpool.token_vault_a, false),
//             AccountMeta::new(token_account_b, false), //token_owner_account_b, false),
//             AccountMeta::new(whirlpool.token_vault_b, false),
//             AccountMeta::new(tick_array_0, false),
//             AccountMeta::new(tick_array_1, false),
//             AccountMeta::new(tick_array_2, false),
//             AccountMeta::new(oracle, false),
//         ],
//     );
//     let transaction = Transaction::new_signed_with_payer(
//         &[swap_ix],
//         Some(&owner.pubkey()),
//         &[&owner],
//         recent_blockhash,
//     );
//     let result = banks_client.process_transaction(transaction).await;
//     match result {
//         Ok(_) => {
//             println!("Transaction processed successfully");
//         }
//         Err(e) => {
//             panic!("Error processing transaction: {:?}", e);
//         }
//     }
// }
