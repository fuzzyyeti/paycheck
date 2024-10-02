mod setup;
mod utils;

use crate::setup::{setup_program, PROGRAM_ID, WHIRLPOOL_ADDRESS};
use paycheck::state::Paycheck;
use solana_program::pubkey::Pubkey;
use solana_program_test::tokio;
use solana_sdk::account::Account;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use paycheck::paycheck_seeds;

#[tokio::test]
async fn test_create_paycheck() {
    let (mut banks_client, payer, recent_blockhash, owner) = setup_program(|p, owner| {
        let (paycheck_address, bump) = Pubkey::find_program_address(
            paycheck_seeds!(WHIRLPOOL_ADDRESS, owner, true),
            &PROGRAM_ID,
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
            a_to_b: true,
        };
        let data = borsh::to_vec(&paycheck).unwrap();
        p.add_account(
            paycheck_address,
            Account {
                owner: PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
                lamports: 100000000,
                data,
            },
        );
    }).await;

    let (paycheck_address, bump) = Pubkey::find_program_address(
        paycheck_seeds!(WHIRLPOOL_ADDRESS, owner.pubkey(), true),
        &PROGRAM_ID,
    );
    let close_paycheck_ix = paycheck::instructions::close_paycheck::create_close_paycheck_ix(
        owner.pubkey(),
        paycheck_address,
    );
    let close_paycheck_tx = Transaction::new_signed_with_payer(
        &[close_paycheck_ix],
        Some(&payer.pubkey()),
        &[&payer, &owner],
        recent_blockhash,
    );
    let paycheck_option = banks_client.get_account(paycheck_address).await.unwrap();
    let rent: u64 = match paycheck_option {
        Some(paycheck) => paycheck.lamports,
        None => panic!("Paycheck account not found"),
    };
    let old_balance = banks_client.get_balance(owner.pubkey()).await.unwrap();
    banks_client
        .process_transaction(close_paycheck_tx)
        .await
        .unwrap();
    assert!(banks_client
        .get_account(paycheck_address)
        .await
        .unwrap()
        .is_none());
    let new_balance = banks_client.get_balance(owner.pubkey()).await.unwrap();
    assert_eq!(old_balance + rent, new_balance);
}
