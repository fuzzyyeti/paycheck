use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use paycheck::consts::PAYCHECK_SEED;
use dotenv::dotenv;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use paycheck::state::Paycheck;
pub const BSOL_MINT: Pubkey = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
fn main() {
    dotenv().ok();
    let rpc_ulr = std::env::var("RPC").expect("RPC must be set");
    let creator = pubkey!("6Zg87oCJg919TC1HGkW2Y9w9RwSEhMWoJTEeecnxsZfw");
    let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
    let client = solana_client::rpc_client::RpcClient::new(rpc_ulr);
    let bot_key_file = std::env::var("BOT_KEY").expect("BOT_KEY must be set");
    let bot_key = solana_sdk::signature::read_keypair_file(&bot_key_file).unwrap();
    println!("{:?}", bot_key.pubkey());

    let paycheck = Pubkey::find_program_address(&[
        PAYCHECK_SEED,
        &whirlpool.to_bytes(),
        &creator.to_bytes()], &paycheck::ID).0;
    let paycheck_account = client.get_account(&paycheck).unwrap();
    let paycheck_data = Paycheck::try_from_slice(paycheck_account.data.as_slice());
    println!("{:?}", paycheck_data);
    let receiver_token_account = get_associated_token_address(
        &bot_key.pubkey(),
        &BSOL_MINT);

    let temp_token_account = Keypair::new();

    // let execute_ix = paycheck::instructions::execute_paycheck_ix(
    //     bot_key.pubkey(),
    //     receiver_token_account,
    //     creator,
    //     whirlpool,
    //     USDC_MINT,
    //     BSOL_MINT,
    //     temp_token_account.pubkey(),
    //
    //
    //
    //
    //
    //     // )
    //     println!("{:?}", paycheck);
}
