use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::BanksClient;

// Not sure why the compiler thinks this is dead code
#[allow(dead_code)]
pub async fn check_balance(banks_client: &mut BanksClient, address: Pubkey, amount: u64) {
    let token_account = banks_client.get_account(address).await.unwrap().unwrap();
    let token_account_data = spl_token::state::Account::unpack(&token_account.data).unwrap();
    assert_eq!(token_account_data.amount, amount);
}

// Not sure why the compiler thinks this is dead code
#[allow(dead_code)]
pub fn get_paycheck_address(owner: &Pubkey, whirlpool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"paycheck", whirlpool.as_ref(), owner.as_ref()],
        &paycheck::ID,
    )
}
