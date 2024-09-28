use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::BanksClient;

pub async fn check_balance(mut banks_client: &mut BanksClient, address: Pubkey, amount: u64) {
    println!("Checking balance for: {:?}", address);
    let token_account = banks_client.get_account(address)
        .await
        .unwrap().unwrap();
    let token_account_data = spl_token::state::Account::unpack(&token_account.data).unwrap();
    println!("token_account amount: {:?}", token_account_data.amount);
    assert_eq!(token_account_data.amount, amount);
}