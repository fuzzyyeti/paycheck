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

pub async fn setup_program<F>(mod_program: F) -> (BanksClient, Keypair, Hash, Keypair)
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
    )
}
