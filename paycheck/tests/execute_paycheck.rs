mod setup;

use borsh::BorshDeserialize;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::tokio;
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, SeedDerivable, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::state::AccountState;
use paycheck::consts::PAYCHECK_SEED;
use paycheck::ID;
use paycheck::instructions::execute_paycheck_ix;
use paycheck::state::Paycheck;
use whirlpools_state::TOKEN_PROGRAM_ID;
use crate::setup::{setup_program, BSOL_MINT, USDC_MINT, WHIRLPOOL_ADDRESS};

// #[tokio::test]
// async fn test_execute_paycheck() {
//     let creator = Pubkey::new_unique();
//     let (paycheck_address, bump) = Pubkey::find_program_address(
//         &[
//             PAYCHECK_SEED,
//             &WHIRLPOOL_ADDRESS.to_bytes(),
//             &creator.to_bytes(),
//         ],
//         &ID,
//     );
//     let (mut banks_client, payer, recent_blockhash, owner, token_account_b) = setup_program(|p| {
//         let paycheck = Paycheck {
//             creator,
//             receiver: Pubkey::new_unique(),
//             increment: 100,
//             last_executed: 0,
//             amount: 10_000_000,
//             whirlpool: WHIRLPOOL_ADDRESS,
//             tip: 50_000,
//             a_to_b: true,
//             bump,
//         };
//         let data = borsh::to_vec(&paycheck).unwrap();
//         p.add_account(
//             paycheck_address,
//             Account {
//                 owner: ID,
//                 executable: false,
//                 rent_epoch: 0,
//                 lamports: 100000000,
//                 data,
//             },
//         );
//
//         let treasury_token_account = spl_token::state::Account {
//             mint: BSOL_MINT,
//             owner: paycheck_address,
//             amount: 1_000_000_000,
//             delegate: COption::None,
//             state: AccountState::Initialized,
//             is_native: COption::None,
//             delegated_amount: 0,
//             close_authority: COption::None,
//         };
//         let treasury_token_account_address =
//             spl_associated_token_account::get_associated_token_address(
//                 &paycheck_address,
//                 &BSOL_MINT,
//             );
//         let mut data: Vec<u8> = vec![0; spl_token::state::Account::get_packed_len()];
//         treasury_token_account.pack_into_slice(&mut data);
//         p.add_account(
//             treasury_token_account_address,
//             Account {
//                 lamports: 100000000,
//                 data,
//                 owner: TOKEN_PROGRAM_ID,
//                 executable: false,
//                 rent_epoch: 0,
//             },
//         );
//     })
//     .await;
//
//     let temp_token_account = Keypair::from_seed(&[1; 32]).unwrap();
//
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
//     let treasury_token_account =
//         spl_associated_token_account::get_associated_token_address(&paycheck_address, &BSOL_MINT);
//
//     let execute_paycheck_ix = execute_paycheck_ix(
//         payer.pubkey(),
//         creator,
//         WHIRLPOOL_ADDRESS,
//         BSOL_MINT,
//         USDC_MINT,
//         treasury_token_account,
//         temp_token_account.pubkey(),
//         whirlpool.token_vault_a,
//         whirlpool.token_vault_b,
//         tick_array_0,
//         tick_array_1,
//         tick_array_2,
//         oracle,
//         true,
//     )
//     .unwrap();
//     let cu_ix =
//         solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(400_000);
//     let tx = Transaction::new_signed_with_payer(
//         &[cu_ix, execute_paycheck_ix],
//         Some(&payer.pubkey()),
//         &[&payer, &temp_token_account],
//         recent_blockhash,
//     );
//     banks_client.process_transaction(tx).await.unwrap();
// }

#[tokio::test]
async fn test_execute_paycheck_reverse() {
    let (mut banks_client, payer, recent_blockhash, owner, token_account_b) = setup_program(|p, owner| {

        let (paycheck_address, bump) = Pubkey::find_program_address(
            &[
                PAYCHECK_SEED,
                &WHIRLPOOL_ADDRESS.to_bytes(),
                &owner.to_bytes(),
            ],
            &ID,
        );
        let paycheck = Paycheck {
            creator: *owner,
            receiver: Pubkey::new_unique(),
            increment: 100,
            last_executed: 0,
            amount: 10_000_000,
            whirlpool: WHIRLPOOL_ADDRESS,
            tip: 50_000,
            bump,
            a_to_b: false,
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

        let treasury_token_account = spl_token::state::Account {
            mint: USDC_MINT,
            owner: paycheck_address,
            amount: 1_000_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let treasury_token_account_address =
            get_associated_token_address(
                &paycheck_address,
                &USDC_MINT,
            );
        let mut data: Vec<u8> = vec![0; spl_token::state::Account::get_packed_len()];
        treasury_token_account.pack_into_slice(&mut data);
        p.add_account(
            treasury_token_account_address,
            Account {
                lamports: 100000000,
                data,
                owner: TOKEN_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }).await;

    let (paycheck_address, bump) = Pubkey::find_program_address(
        &[
            PAYCHECK_SEED,
            &WHIRLPOOL_ADDRESS.to_bytes(),
            &owner.pubkey().to_bytes(),
        ],
        &ID,
    );

    let temp_token_account = Keypair::from_seed(&[1; 32]).unwrap();
    let receiver_token_account = get_associated_token_address(&owner.pubkey(), &BSOL_MINT);
    let payer_token_account = get_associated_token_address(&payer.pubkey(), &BSOL_MINT);
    println!("Payer token account: {:?}", payer_token_account);
    let create_payer_account_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &BSOL_MINT,
        &spl_token::id(),
    );

    let create_accounts_tx = Transaction::new_signed_with_payer(
        &[create_payer_account_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(create_accounts_tx).await.unwrap();


    let oracle = Pubkey::find_program_address(
        &[b"oracle", WHIRLPOOL_ADDRESS.as_ref()],
        &whirlpools_state::ID,
    )
        .0;
    let whirlpool = whirlpools_state::Whirlpool::try_from_slice(
        &banks_client
            .get_account(WHIRLPOOL_ADDRESS)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
        .unwrap();

    let index_spacing = (whirlpool.tick_spacing as i32) * 88;
    let start_tick_index =
        whirlpool.tick_current_index - (whirlpool.tick_current_index % index_spacing);
    let tick_array_0 = Pubkey::find_program_address(
        &[
            b"tick_array",
            WHIRLPOOL_ADDRESS.as_ref(),
            start_tick_index.to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    )
        .0;
    let tick_array_1 = Pubkey::find_program_address(
        &[
            b"tick_array",
            WHIRLPOOL_ADDRESS.as_ref(),
            (start_tick_index - index_spacing).to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    )
        .0;
    let tick_array_2 = Pubkey::find_program_address(
        &[
            b"tick_array",
            WHIRLPOOL_ADDRESS.as_ref(),
            (start_tick_index - index_spacing * 2)
                .to_string()
                .as_bytes(),
        ],
        &whirlpools_state::ID,
    )
        .0;
    let treasury_token_account =
        spl_associated_token_account::get_associated_token_address(&paycheck_address, &USDC_MINT);

    let execute_paycheck_ix = execute_paycheck_ix(
        payer.pubkey(),
        owner.pubkey(),
        WHIRLPOOL_ADDRESS,
        USDC_MINT,
        BSOL_MINT,
        treasury_token_account,
        temp_token_account.pubkey(),
        whirlpool.token_vault_a,
        whirlpool.token_vault_b,
        tick_array_0,
        tick_array_1,
        tick_array_2,
        oracle,
        false,
    ).unwrap();
    let cu_ix =
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let tx = Transaction::new_signed_with_payer(
        &[cu_ix.clone(), execute_paycheck_ix.clone()],
        Some(&payer.pubkey()),
        &[&payer, &temp_token_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[cu_ix, execute_paycheck_ix],
        Some(&payer.pubkey()),
        &[&payer, &temp_token_account],
        blockhash,
    );
    banks_client.process_transaction(tx2).await.unwrap();
}
