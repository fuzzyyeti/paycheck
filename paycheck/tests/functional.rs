use borsh::BorshDeserialize;
use once_cell::sync::Lazy;
use paycheck::consts::PAYCHECK_SEED;
use paycheck::instructions::create_paycheck::CreatePaycheckArgs;
use paycheck::instructions::execute_paycheck_ix;
use paycheck::state::Paycheck;
use paycheck::ID;
use solana_program::hash::Hash;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, tokio, BanksClient, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, SeedDerivable, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::state::AccountState;
use std::str::FromStr;
use whirlpools_state::{SwapArgs, TOKEN_PROGRAM_ID, USDC_MINT};

static PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str("54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4").unwrap());

const WHIRLPOOL_ADDRESS: Pubkey = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
const BSOL_MINT: Pubkey = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
// #[tokio::test]
// async fn try_swap() {
//     let program_id = *PROGRAM_ID;
//     let (mut banks_client, payer, recent_blockhash, owner, token_account_b) = setup_program(|p| p).await;
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
//             .data[8..],
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
//     let swap_discriminator: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
//     let input_args = SwapArgs {
//         swap_discriminator,
//         amount: 1000,
//         other_amount_threshold: 0,
//         sqrt_price_limit: 0,
//         amount_specified_is_input: true,
//         a_to_b: true,
//     };
//
//     let swap_ix = Instruction::new_with_borsh(
//         whirlpools_state::ID,
//         &input_args,
//         vec![
//             AccountMeta::new_readonly(
//                 pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
//                 false,
//             ),
//             AccountMeta::new(owner.pubkey(), true),
//             AccountMeta::new(WHIRLPOOL_ADDRESS, false),
//             AccountMeta::new(token_account_address, false),
//             AccountMeta::new(whirlpool.token_vault_a, false),
//             AccountMeta::new(token_account_b, false), //token_owner_account_b, false),
//             AccountMeta::new(whirlpool.token_vault_b, false),
//             AccountMeta::new(tick_array_0, false),
//             AccountMeta::new(tick_array_1, false),
//             AccountMeta::new(tick_array_2, false),
//             AccountMeta::new_readonly(oracle, false),
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
// #[tokio::test]
// async fn test_create_paycheck() {
//     let program_id = *PROGRAM_ID;
//     println!("program_id: {:?}", program_id);
//     let (mut banks_client, payer, recent_blockhash, _, _) = setup_program(|p| p).await;
//     let target_mint = Pubkey::new_unique();
//     let whirlpool = Pubkey::new_unique();
//     let receiver = Pubkey::new_unique();
//     let args = CreatePaycheckArgs {
//         receiver,
//         start_date: 8,
//         increment: 8,
//         amount: 8,
//         whirlpool,
//         target_mint,
//         tip: 8,
//     };
//     let create_paycheck_ix =
//         paycheck::instructions::create_paycheck::create_paycheck_ix(payer.pubkey(), args.clone())
//             .unwrap();
//     println!("create_config_ix: {:?}", create_paycheck_ix);
//     let transaction = Transaction::new_signed_with_payer(
//         &[create_paycheck_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );
//     let result = banks_client.process_transaction(transaction).await;
//
//     match result {
//         Ok(_) => {
//             println!("Transaction processed successfully");
//
//             // Find the Paycheck account PDA with correct seeds
//             let (paycheck_pda, _) = Pubkey::find_program_address(
//                 &[
//                     b"paycheck",
//                     args.whirlpool.as_ref(),
//                     payer.pubkey().as_ref(),
//                 ],
//                 &program_id,
//             );
//
//             // Fetch the Paycheck account data
//             let paycheck_account = banks_client
//                 .get_account(paycheck_pda)
//                 .await
//                 .expect("Failed to fetch Paycheck account");
//
//             if let Some(account) = paycheck_account {
//                 let paycheck = Paycheck::try_from_slice(&account.data)
//                     .expect("Failed to deserialize Paycheck account");
//
//                 // Verify the Paycheck account data
//                 assert_eq!(paycheck.receiver, args.receiver);
//                 assert_eq!(paycheck.start_date, args.start_date);
//                 assert_eq!(paycheck.increment, args.increment);
//                 assert_eq!(paycheck.amount, args.amount);
//                 assert_eq!(paycheck.whirlpool, args.whirlpool);
//                 assert_eq!(paycheck.target_mint, args.target_mint);
//                 assert_eq!(paycheck.tip, args.tip);
//                 assert!(paycheck.is_enabled);
//
//                 println!("Paycheck account configured correctly");
//             } else {
//                 panic!("Paycheck account not found");
//             }
//         }
//         Err(e) => {
//             panic!("Error processing transaction: {:?}", e);
//         }
//     }
// }

#[tokio::test]
async fn test_execute_paycheck() {
    let creator = Pubkey::new_unique();
    let (paycheck_address, bump) = Pubkey::find_program_address(
        &[
            PAYCHECK_SEED,
            &WHIRLPOOL_ADDRESS.to_bytes(),
            &creator.to_bytes(),
        ],
        &ID,
    );
    let (mut banks_client, payer, recent_blockhash, owner, token_account_b) =
        setup_program(|mut p| {
            let paycheck = Paycheck {
                creator,
                receiver: Pubkey::new_unique(),
                start_date: 100,
                increment: 100,
                amount: 10_000_000,
                whirlpool: WHIRLPOOL_ADDRESS,
                target_mint: USDC_MINT,
                tip: 50_000,
                is_enabled: true,
                bump,
            };
            let data = borsh::to_vec(&paycheck).unwrap();
            p.add_account(
                paycheck_address,
                Account {
                    owner: ID,
                    executable: false,
                    rent_epoch: 0,
                    lamports: 100000000,
                    data,
                },
            );
        })
        .await;

    let temp_token_account = Keypair::from_seed(&[1; 32]).unwrap();
    let execute_paycheck_ix = execute_paycheck_ix(
        payer.pubkey(),
        creator,
        WHIRLPOOL_ADDRESS,
        USDC_MINT,
        temp_token_account.pubkey(),
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[execute_paycheck_ix],
        Some(&payer.pubkey()),
        &[&payer, &temp_token_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
}

async fn setup_program<F>(mod_program: F) -> (BanksClient, Keypair, Hash, Keypair, Pubkey)
where
    F: FnOnce(&mut ProgramTest) -> (),
{
    let mut program_test = ProgramTest::new(
        "paycheck",
        *PROGRAM_ID,
        processor!(paycheck::processor::process_instruction),
    );
    program_test.add_program("whirlpool_program", whirlpools_state::ID, None);
    program_test.add_account_with_file_data(
        WHIRLPOOL_ADDRESS,
        5435760,
        pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        "./tests/data/whirlpool.bin",
    );
    program_test.add_account_with_file_data(
        pubkey!("CnnLoEyGjS1Bwhnc5fCHHoU6fLQ7zfxp7UAqgoBX27QL"),
        2039280,
        pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        "./tests/data/token_vault_a.bin",
    );
    program_test.add_account_with_file_data(
        pubkey!("FAehFHnQqqP6Mq9yY6ofFKPFDdz1K5dK2FsrTTf3o4Gq"),
        2039280,
        pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        "./tests/data/token_vault_b.bin",
    );
    program_test.add_account_with_file_data(
        pubkey!("BMGf4rTHvJsXiGPqur4NcEqT4iizBu8kqAvREae5VLXt"),
        70407360,
        pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        "./tests/data/tick_array_0.bin",
    );
    program_test.add_account_with_file_data(
        pubkey!("4F2Hn2R9guCefV9jdXqkUKd2WhtLPVH2dWr2wF3mQceh"),
        70407360,
        pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        "./tests/data/tick_array_1.bin",
    );
    program_test.add_account_with_file_data(
        pubkey!("G3JXJabcbA6dHNDVkrrP88v5L8jkW5SE3CrU3fNiZ5LH"),
        70407360,
        pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        "./tests/data/tick_array_2.bin",
    );
    program_test.add_account_with_file_data(
        BSOL_MINT,
        4118320394,
        spl_token::id(),
        "./tests/data/bsol_mint.bin",
    );
    program_test.add_account_with_file_data(
        USDC_MINT,
        4118320394,
        spl_token::id(),
        "./tests/data/usdc_mint.bin",
    );

    mod_program(&mut program_test);

    let owner = Keypair::new();
    let token_account_a = spl_token::state::Account {
        mint: BSOL_MINT,
        owner: owner.pubkey(),
        amount: 1000000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let token_account_address =
        spl_associated_token_account::get_associated_token_address(&owner.pubkey(), &BSOL_MINT);
    let mut data: Vec<u8> = vec![0; spl_token::state::Account::get_packed_len()];
    token_account_a.pack_into_slice(&mut data);

    program_test.add_account(
        token_account_address,
        Account {
            lamports: 100000000,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let token_b_account_onwer = Pubkey::new_unique();
    let token_account_b = spl_token::state::Account {
        mint: USDC_MINT,
        /// The owner of this account.
        owner: token_b_account_onwer,
        /// The amount of tokens this account holds.
        amount: 1000000,
        /// If `delegate` is `Some` then `delegated_amount` represents
        /// the amount authorized by the delegate
        delegate: COption::None,
        /// The account's state
        state: AccountState::Initialized,
        /// If is_native.is_some, this is a native token, and the value logs the
        /// rent-exempt reserve. An Account is required to be rent-exempt, so
        /// the value is used by the Processor to ensure that wrapped SOL
        /// accounts do not drop below this threshold.
        is_native: COption::None,
        /// The amount delegated
        delegated_amount: 0,
        /// Optional authority to close the account.
        close_authority: COption::None,
    };
    let token_account_b_address = spl_associated_token_account::get_associated_token_address(
        &token_b_account_onwer,
        &USDC_MINT,
    );
    let mut data_b: Vec<u8> = vec![0; spl_token::state::Account::get_packed_len()];
    token_account_b.pack_into_slice(&mut data_b);
    program_test.add_account(
        token_account_b_address,
        Account {
            lamports: 100000000,
            data: data_b,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 100000000,
            data: vec![0; 0],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    (
        banks_client,
        payer,
        recent_blockhash,
        owner,
        token_account_b_address,
    )
}
