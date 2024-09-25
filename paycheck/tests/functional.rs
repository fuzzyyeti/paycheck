use borsh::BorshDeserialize;
use paycheck::consts::PAYCHECK_SEED;
use paycheck::instructions::{execute_paycheck_ix, CreatePaycheckArgs};
use paycheck::state::Paycheck;
use paycheck::ID;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::{tokio};
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, SeedDerivable, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::state::AccountState;
use std::str::FromStr;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey;
use whirlpools_state::{SwapArgs, TOKEN_PROGRAM_ID};
use crate::setup::{setup_program, BSOL_MINT, PROGRAM_ID, USDC_MINT, WHIRLPOOL_ADDRESS};

mod setup;

const SWAP_DISCRIMINATOR: [u8; 8] = [ 0x2b, 0x04, 0xed, 0x0b, 0x1a, 0xc9, 0x1e, 0x62];

#[tokio::test]
async fn try_swap() {
    let program_id = PROGRAM_ID;
    let (mut banks_client, payer, recent_blockhash, owner, token_account_b) = setup_program(|p| {}).await;
    let token_account_address =
        spl_associated_token_account::get_associated_token_address(&owner.pubkey(), &BSOL_MINT);
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

    let input_args = SwapArgs {
        swap_discriminator: SWAP_DISCRIMINATOR,
        amount: 1000,
        other_amount_threshold: u64::MAX,
        sqrt_price_limit: 0,
        amount_specified_is_input: false,
        a_to_b: true,
        remaining_accounts_info: None,
    };

    let swap_ix = Instruction::new_with_borsh(
        whirlpools_state::ID,
        &input_args,
        vec![
            AccountMeta::new_readonly(
                spl_token::id(),
                false,
            ),
            AccountMeta::new_readonly(
                spl_token::id(),
                false,
            ),
            AccountMeta::new_readonly(
                pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
               false
            ),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(WHIRLPOOL_ADDRESS, false),
            AccountMeta::new(BSOL_MINT, false),
            AccountMeta::new(USDC_MINT, false),
            AccountMeta::new(token_account_address, false),
            AccountMeta::new(whirlpool.token_vault_a, false),
            AccountMeta::new(token_account_b, false), //token_owner_account_b, false),
            AccountMeta::new(whirlpool.token_vault_b, false),
            AccountMeta::new(tick_array_0, false),
            AccountMeta::new(tick_array_1, false),
            AccountMeta::new(tick_array_2, false),
            AccountMeta::new(oracle, false),
        ],
    );
    let transaction = Transaction::new_signed_with_payer(
        &[swap_ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash,
    );
    let result = banks_client.process_transaction(transaction).await;
    match result {
        Ok(_) => {
            println!("Transaction processed successfully");
        }
        Err(e) => {
            panic!("Error processing transaction: {:?}", e);
        }
    }
}
#[tokio::test]
async fn test_create_paycheck() {
    let program_id = PROGRAM_ID;
    println!("program_id: {:?}", program_id);
    let (mut banks_client, payer, recent_blockhash, _, _) = setup_program(|p| {}).await;
    let target_mint = Pubkey::new_unique();
    let whirlpool = Pubkey::new_unique();
    let receiver = Pubkey::new_unique();
    let args = CreatePaycheckArgs {
        receiver,
        start_date: 8,
        increment: 8,
        amount: 8,
        whirlpool,
        target_mint,
        tip: 8,
    };
    let create_paycheck_ix =
        paycheck::instructions::create_paycheck::create_paycheck_ix(payer.pubkey(), args.clone())
            .unwrap();
    println!("create_config_ix: {:?}", create_paycheck_ix);
    let transaction = Transaction::new_signed_with_payer(
        &[create_paycheck_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    let result = banks_client.process_transaction(transaction).await;

    match result {
        Ok(_) => {
            println!("Transaction processed successfully");

            // Find the Paycheck account PDA with correct seeds
            let (paycheck_pda, _) = Pubkey::find_program_address(
                &[
                    b"paycheck",
                    args.whirlpool.as_ref(),
                    payer.pubkey().as_ref(),
                ],
                &program_id,
            );

            // Fetch the Paycheck account data
            let paycheck_account = banks_client
                .get_account(paycheck_pda)
                .await
                .expect("Failed to fetch Paycheck account");

            if let Some(account) = paycheck_account {
                let paycheck = Paycheck::try_from_slice(&account.data)
                    .expect("Failed to deserialize Paycheck account");

                // Verify the Paycheck account data
                assert_eq!(paycheck.receiver, args.receiver);
                assert_eq!(paycheck.start_date, args.start_date);
                assert_eq!(paycheck.increment, args.increment);
                assert_eq!(paycheck.amount, args.amount);
                assert_eq!(paycheck.whirlpool, args.whirlpool);
                assert_eq!(paycheck.target_mint, args.target_mint);
                assert_eq!(paycheck.tip, args.tip);
                assert!(paycheck.is_enabled);

                println!("Paycheck account configured correctly");
            } else {
                panic!("Paycheck account not found");
            }
        }
        Err(e) => {
            panic!("Error processing transaction: {:?}", e);
        }
    }
}

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

            let treasury_token_account = spl_token::state::Account {
                mint: BSOL_MINT,
                owner: paycheck_address,
                amount: 1_000_000_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            };
            let treasury_token_account_address =
                spl_associated_token_account::get_associated_token_address(&paycheck_address, &BSOL_MINT);
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
        })
        .await;

    let temp_token_account = Keypair::from_seed(&[1; 32]).unwrap();

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
    ).unwrap();

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
    ).0;
    let treasury_token_account = spl_associated_token_account::get_associated_token_address(
        &paycheck_address,
        &BSOL_MINT,
    );

    println!("treasury_token_account outside: {:?}", treasury_token_account);
    let execute_paycheck_ix = execute_paycheck_ix(
        payer.pubkey(),
        creator,
        WHIRLPOOL_ADDRESS,
        BSOL_MINT,
        USDC_MINT,
        treasury_token_account,
        temp_token_account.pubkey(),
        whirlpool.token_vault_a,
        whirlpool.token_vault_b,
        tick_array_0,
        tick_array_1,
        tick_array_2,
        oracle,
    )
    .unwrap();
    let cu_ix = solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let tx = Transaction::new_signed_with_payer(
        &[cu_ix, execute_paycheck_ix],
        Some(&payer.pubkey()),
        &[&payer, &temp_token_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
}

