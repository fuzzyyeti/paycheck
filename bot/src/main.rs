use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use dotenv::dotenv;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use paycheck::paycheck_seeds;
use paycheck::state::Paycheck;
pub const BSOL_MINT: Pubkey = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
fn main() {
    dotenv().ok();
    let rpc_ulr = std::env::var("RPC").expect("RPC must be set");
    let creator = pubkey!("6Zg87oCJg919TC1HGkW2Y9w9RwSEhMWoJTEeecnxsZfw");
    let whirlpool_address = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
    let client = solana_client::rpc_client::RpcClient::new(rpc_ulr);
    let bot_key_file = std::env::var("BOT_KEY").expect("BOT_KEY must be set");
    let bot_key = solana_sdk::signature::read_keypair_file(&bot_key_file).unwrap();
    println!("{:?}", bot_key.pubkey());

    let whirlpool_account = client.get_account(&whirlpool_address).unwrap();
    let whirlpool = whirlpools_state::Whirlpool::try_from_slice(
        whirlpool_account.data.as_slice()
    ).unwrap();


    let paycheck_address = Pubkey::find_program_address(
        paycheck_seeds!(
            whirlpool_address,
            creator,
            true
        ),
        &paycheck::ID).0;
    println!("{:?}", paycheck_address);
    let paycheck_account = client.get_account(&paycheck_address).unwrap();
    let paycheck = Paycheck::try_from_slice(paycheck_account.data.as_slice()).unwrap();
    let index_spacing = (whirlpool.tick_spacing as i32) * 88;
    let start_tick_index =
        whirlpool.tick_current_index - (whirlpool.tick_current_index % index_spacing);
    let calc_next_index = |a: i32, b: i32| if paycheck.a_to_b { a - b } else { a + b };

    let tick_array_0 = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_address.as_ref(),
            start_tick_index.to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    )
        .0;
    let tick_array_1 = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_address.as_ref(),
            calc_next_index(start_tick_index, index_spacing).to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    ).0;
    let tick_array_2 = Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool_address.as_ref(),
            calc_next_index(start_tick_index,index_spacing * 2)
                .to_string()
                .as_bytes(),
        ],
        &whirlpools_state::ID,
    ).0;
    println!("{:?}", paycheck);
    let receiver_token_account_address = get_associated_token_address(
        &paycheck.receiver,
        &USDC_MINT);

    let receiver_token_account = client.get_account(&receiver_token_account_address);

    match receiver_token_account {
        Ok(_) => {
            println!("Receiver token account exists");
        }
        Err(_) => {
            let create_receiver_token_account_ix = create_associated_token_account(
                &bot_key.pubkey(),
                &paycheck.receiver,
                &USDC_MINT,
                &spl_token::id(),
            );
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let ix = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[create_receiver_token_account_ix],
                Some(&bot_key.pubkey()),
                &[&bot_key],
                recent_blockhash,
            );
            let signature = client.send_and_confirm_transaction(&ix).unwrap();
            println!("{:?}", signature);
        }
    }

    let payer_token_account_address = get_associated_token_address(
        &bot_key.pubkey(),
        &USDC_MINT);
    println!("{:?}", payer_token_account_address);
    let payer_token_account = client.get_account(&payer_token_account_address);
    match payer_token_account {
        Ok(_) => {
            println!("Payer token account exists");
        }
        Err(_) => {
            let create_payer_token_account_ix = create_associated_token_account(
                &bot_key.pubkey(),
                &bot_key.pubkey(),
                &USDC_MINT,
                &spl_token::id(),
            );
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let ix = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[create_payer_token_account_ix],
                Some(&bot_key.pubkey()),
                &[&bot_key],
                recent_blockhash,
            );
            let signature = client.send_and_confirm_transaction(&ix).unwrap();
            println!("{:?}", signature);
        }
    }

    let temp_token_account = Keypair::new();

    let oracle = Pubkey::find_program_address(
        &[b"oracle", whirlpool_address.as_ref()],
        &whirlpools_state::ID,
    ).0;

    let treasury_token_account = get_associated_token_address(
        &paycheck.creator,
        &BSOL_MINT);

    println!("treasury_token_account: {:?}", treasury_token_account);

    let execute_ix = paycheck::instructions::execute_paycheck_ix(
        bot_key.pubkey(),
        receiver_token_account_address,
        creator,
        whirlpool_address,
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
        paycheck.a_to_b).unwrap();

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let ix = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&bot_key.pubkey()),
        &[&bot_key, &temp_token_account],
        recent_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&ix).unwrap();
    println!("{:?}", signature);
}
