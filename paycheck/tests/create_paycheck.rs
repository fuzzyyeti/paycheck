mod setup;

use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use solana_program_test::tokio;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use paycheck::instructions::CreatePaycheckArgs;
use paycheck::state::Paycheck;
use crate::setup::{setup_program, PROGRAM_ID};

#[tokio::test]
async fn test_create_paycheck() {
    let program_id = PROGRAM_ID;
    println!("program_id: {:?}", program_id);
    let (mut banks_client, payer, recent_blockhash, _, _) = setup_program(|p| {}).await;
    let whirlpool = Pubkey::new_unique();
    let receiver = Pubkey::new_unique();
    let args = CreatePaycheckArgs {
        receiver,
        increment: 8,
        amount: 8,
        whirlpool,
        tip: 8,
        a_to_b: true,
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
                assert_eq!(paycheck.increment, args.increment);
                assert_eq!(paycheck.amount, args.amount);
                assert_eq!(paycheck.whirlpool, args.whirlpool);
                assert_eq!(paycheck.tip, args.tip);

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

