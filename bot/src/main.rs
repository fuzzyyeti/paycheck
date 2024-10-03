use borsh::BorshDeserialize;
use chrono::Utc;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use dotenv::dotenv;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use paycheck::state::Paycheck;

fn main() {
    dotenv().ok();
    let rpc_ulr = std::env::var("RPC").expect("RPC must be set");
    let creator = pubkey!("6Zg87oCJg919TC1HGkW2Y9w9RwSEhMWoJTEeecnxsZfw");
    let whirlpool_address = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
    let client = solana_client::rpc_client::RpcClient::new(rpc_ulr);
    let bot_key_file = std::env::var("BOT_KEY").expect("BOT_KEY must be set");
    let bot_key = solana_sdk::signature::read_keypair_file(&bot_key_file).unwrap();
    println!("{:?}", bot_key.pubkey());


    let binary_wp_address = whirlpool_address.to_bytes().to_vec();
    println!("bin length {:?}", binary_wp_address.len());
    let paycheck_addresses = match client.get_program_accounts_with_config(
        &paycheck::ID,
        solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(vec![
               // Check that it is SolBlze/USDC
                RpcFilterType::Memcmp(
                solana_client::rpc_filter::Memcmp::new_raw_bytes(
                    88,
                    binary_wp_address
                )),
                // Check that it is a_b
                RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_raw_bytes(
                        128,
                        1u8.to_le_bytes().to_vec()
                    )
                )
            ]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                ..RpcAccountInfoConfig::default()
            },
            with_context: None,
        }
    ) {
        Ok(accounts) => accounts,
        Err(e) => {
            println!("{:?}", e);
            println!("No paycheck accounts found");
            return;
        }
    };

    for (paycheck_address, account) in paycheck_addresses {
        let paycheck = Paycheck::try_from_slice(&account.data);
        match paycheck {
            Ok(paycheck) => {
                let current_time = Utc::now().timestamp();
                if paycheck.last_executed + paycheck.increment < current_time {
                    println!("Executing paycheck {:?}", paycheck_address);
                    execute_paycheck(paycheck, &client, &bot_key);
                } else {
                    println!("Increment not passed {:?}", paycheck_address);
                }
            }
            Err(e) => {
                println!("Couldn't deserialize the paycheck {:?}", e);
            }
        }
    }


}

fn execute_paycheck(paycheck: Paycheck, client: &RpcClient, bot_key: &Keypair) {
    let whirlpool_account = client.get_account(&paycheck.whirlpool).unwrap();
    let whirlpool = whirlpools_state::Whirlpool::try_from_slice(
        whirlpool_account.data.as_slice()
    ).unwrap();
    let index_spacing = (whirlpool.tick_spacing as i32) * 88;
    let start_tick_index =
        whirlpool.tick_current_index - (whirlpool.tick_current_index % index_spacing);
    let calc_next_index = |a: i32, b: i32| if paycheck.a_to_b { a - b } else { a + b };
    let (input_mint, output_mint) = if paycheck.a_to_b {
        (whirlpool.token_mint_a, whirlpool.token_mint_b)
    } else {
        (whirlpool.token_mint_b, whirlpool.token_mint_a)
    };
    println!("{:?}", input_mint);
    println!("{:?}", output_mint);

    let tick_array_0 = Pubkey::find_program_address(
        &[
            b"tick_array",
            paycheck.whirlpool.as_ref(),
            start_tick_index.to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    )
        .0;
    let tick_array_1 = Pubkey::find_program_address(
        &[
            b"tick_array",
            paycheck.whirlpool.as_ref(),
            calc_next_index(start_tick_index, index_spacing).to_string().as_bytes(),
        ],
        &whirlpools_state::ID,
    ).0;
    let tick_array_2 = Pubkey::find_program_address(
        &[
            b"tick_array",
            paycheck.whirlpool.as_ref(),
            calc_next_index(start_tick_index, index_spacing * 2)
                .to_string()
                .as_bytes(),
        ],
        &whirlpools_state::ID,
    ).0;
    println!("{:?}", paycheck);
    let receiver_token_account_address = get_associated_token_address(
        &paycheck.receiver,
        &output_mint);

    let receiver_token_account = client.get_account(&receiver_token_account_address);

    match receiver_token_account {
        Ok(_) => {
            println!("Receiver token account exists");
        }
        Err(_) => {
            let create_receiver_token_account_ix = create_associated_token_account(
                &bot_key.pubkey(),
                &paycheck.receiver,
                &output_mint,
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
        &output_mint);
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
                &output_mint,
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
        &[b"oracle", paycheck.whirlpool.as_ref()],
        &whirlpools_state::ID,
    ).0;

    let treasury_token_account_address = get_associated_token_address(
        &paycheck.creator,
        &input_mint);

    let treasury_token_account=  client.get_account(&treasury_token_account_address);

    match treasury_token_account {
        Ok(_) => {
            println!("Treasury token account exists");
        }
        Err(_) => {
            let create_treasury_token_account_ix = create_associated_token_account(
                &bot_key.pubkey(),
                &paycheck.creator,
                &input_mint,
                &spl_token::id(),
            );
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let ix = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[create_treasury_token_account_ix],
                Some(&bot_key.pubkey()),
                &[&bot_key],
                recent_blockhash,
            );
            let signature = client.send_and_confirm_transaction(&ix).unwrap();
            println!("{:?}", signature);
        }
    }

    let execute_ix = paycheck::instructions::execute_paycheck_ix(
        bot_key.pubkey(),
        receiver_token_account_address,
        paycheck.creator,
        paycheck.whirlpool,
        input_mint,
        output_mint,
        treasury_token_account_address,
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
