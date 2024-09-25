use solana_program::hash::Hash;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, Signer};
use spl_token::state::AccountState;
use whirlpools_state::TOKEN_PROGRAM_ID;

pub const PROGRAM_ID: Pubkey = pubkey!("54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4");

pub const WHIRLPOOL_ADDRESS: Pubkey = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
pub const BSOL_MINT: Pubkey = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub async fn setup_program<F>(mod_program: F) -> (BanksClient, Keypair, Hash, Keypair, Pubkey)
where
    F: FnOnce(&mut ProgramTest, &Pubkey) -> (),
{
    let mut program_test = ProgramTest::new(
        "paycheck",
        PROGRAM_ID,
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

    let owner = Keypair::new();
    mod_program(&mut program_test, &owner.pubkey());

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
    println!("Receiver owner: {:?}", owner.pubkey());
    println!("Token account address: {:?}", token_account_address);
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

    let (banks_client, payer, recent_blockhash) = program_test.start().await;
    (
        banks_client,
        payer,
        recent_blockhash,
        owner,
        token_account_b_address,
    )
}
