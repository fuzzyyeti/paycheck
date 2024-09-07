use std::str::FromStr;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::processor::process_instruction;
use paycheck::instructions::create_config::ConfigArgs;

#[tokio::test]
async fn test_user_claim() {

    let program_id = Pubkey::from_str("54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4").unwrap();
    let source_pubkey = Pubkey::new_unique();
    let prize_mint = Pubkey::new_unique();
    let mut program_test =
        ProgramTest::new("paycheck", program_id, processor!(process_instruction));
    let keypair = Keypair::new();
    program_test.add_account(
        keypair.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let args = ConfigArgs {
        admin: payer.pubkey(),
    };
    let create_config_ix = paycheck::instructions::create_config::create_config_ix(
        &program_id,
        args
    );
    let  transaction = Transaction::new_signed_with_payer(
        &[create_config_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    let result = banks_client.process_transaction(transaction).await;
    panic!("{:?}", result);
    match result {
        Ok(_) => {
            println!("Success");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
