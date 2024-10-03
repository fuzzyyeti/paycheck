#![allow(non_snake_case)]

use std::str::FromStr;
use crate::hooks::use_wallet_adapter::{
    invoke_signature, use_wallet_adapter, use_wallet_adapter_provider, InvokeSignatureStatus,
    WalletAdapter,
};
use borsh::BorshDeserialize;
use chrono::{NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use hooks::use_wallet_adapter::use_balance;
use paycheck::paycheck_seeds;
use paycheck::state::Paycheck;
use solana_client_wasm::{
    solana_sdk::{native_token::lamports_to_sol, pubkey::Pubkey, transaction::Transaction},
    WasmClient,
};
use solana_extra_wasm::program::spl_associated_token_account::get_associated_token_address;
use solana_extra_wasm::program::spl_associated_token_account::instruction::create_associated_token_account;
use solana_extra_wasm::program::spl_token;
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::pubkey;
use tracing::Level;

mod hooks;

pub const RPC_URL: &str = "http://127.0.0.1:8899";

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    use_wallet_adapter_provider();
    rsx! {
        div {
            class: "app-container",
            div {
                class: "anton-sc-regular paycheck-title",
                h1 { "Paycheck" }
            }
            div {
                class: "top-right",
                MountWalletAdapter {}
            }
            div {
                class: "stacked",
                CreatePaycheck {}
                ShowPaychecks {}
            }
        }
    }
}

fn MountWalletAdapter() -> Element {
    let _ = use_future(move || async move {
        let eval = eval(
            r#"
                let mount = window.MountWalletAdapter;
                console.log(mount);
                mount();
                return
            "#,
        );
        let _ = eval.await;
    });
    rsx!(nav {
        id: "dioxus-wallet-adapter"
    })
}

fn ShowPaychecks() -> Element {
    let wallet_adapter = use_wallet_adapter();
    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                tracing::info!("connected the show paychecks {:?}", pubkey);
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let (paycheck_address, _) = Pubkey::find_program_address(
                    paycheck_seeds!(whirlpool, pubkey, true),
                    &paycheck::ID,
                );

                let client = WasmClient::new(RPC_URL);
                let paycheck_account = client.get_account(&paycheck_address).await;
                tracing::info!("paycheck account: {:?}", paycheck_account);
                let result = match paycheck_account {
                    Ok(account) => {
                        tracing::info!("paycheck account 2 : {:?}", account);
                        let data: Paycheck = Paycheck::try_from_slice(&account.data).unwrap();
                        Some(data)
                    }
                    Err(_) => None,
                };
                dioxus_logger::tracing::info!("paycheck data: {:?}", result);

                result
            }
        }
    });
    let increment_days = tx.read().as_ref()
        .and_then(|opt| opt.as_ref()
            .map(|paycheck| paycheck.increment / (24 * 60 * 60)));

    let last_executed_date = tx.read().as_ref()
        .and_then(|opt| opt.as_ref().map(|paycheck| {
            match Utc.timestamp_opt(paycheck.last_executed, 0) {
                chrono::LocalResult::Single(datetime) => datetime.format("%m-%d-%y").to_string(),
                _ => "Invalid date".to_string(),
            }
        })).unwrap_or_else(|| "Invalid date".to_string());

    let ui_amount = tx.read().as_ref()
        .and_then(|opt| opt.as_ref().map(
            |paycheck| (paycheck.amount as f64) / 1_000_000.0)).unwrap_or_else(|| {
        tracing::error!("Error getting paycheck amount");
        0.0
    });

    let ui_tip = tx.read().as_ref()
        .and_then(|opt| opt.as_ref().map(
            |paycheck| (paycheck.tip as f64) / 1_000_000.0)).unwrap_or_else(|| {
        tracing::error!("Error getting paycheck tip");
        0.0
    });

    rsx! {
        div {
            if let Some(Some(paycheck)) = &*tx.read() {
                div {
                    class: "paycheck-card",
                    h2 { "Paycheck Active" }
                    p { "Converting your ORE to USDC and sending to"}
                    p { "{paycheck.receiver}" }
                    p {
                        style: "margin-top: 3rem;",
                        "Increment: {increment_days.unwrap()} days" }
                    p { "Last Executed: {last_executed_date}" }
                    p { "Amount: {ui_amount} USDC" }
                    p { "Tip: {ui_tip} USDC" }
                    div {
                        class: "trash-icon",
                        i { class: "fas fa-trash" }
                    }
                }
            } else {
                p { "Loading paycheck data..." }
            }
        }
    }
}

fn CreatePaycheck() -> Element {
    let status = use_signal(|| InvokeSignatureStatus::Start);
    let wallet_adapter = use_wallet_adapter();
    let mut selected_days = use_signal(|| 1); // Default to 1 day
    let mut amount = use_signal(|| 5.0); // Default amount
    let mut tip = use_signal(|| 0.10); //
    let mut receiver = use_signal(|| String::new());

    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let rpc = WasmClient::new(RPC_URL);
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let bsol = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
                let usdc = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
                let receiver_wallet = pubkey!("2qztb9WG2sGGArmKitC7wwCDMwHTLSVSyqvQCQGY4da5");
                let increment_seconds = selected_days * 24 * 60 * 60; // Convert days to seconds
                let amount_usdc = (amount * 1_000_000.0) as u64;
                let tip_usdc = (tip * 1_000_000.0) as u64;

                dioxus_logger::tracing::info!("Trying to get receiver {:?}", &receiver.read());
                let receiver_pubkey = Pubkey::from_str(&receiver.read()).unwrap_or(pubkey);
                // dioxus_logger::tracing::info!("Receiver pubkey: {:?}", receiver_pubkey);
                // let receiver_token_address = get_associated_token_address(&receiver_pubkey, &usdc);

                let ix = paycheck::instructions::create_paycheck::create_paycheck_ix(
                    pubkey,
                    paycheck::instructions::create_paycheck::CreatePaycheckArgs {
                        receiver: receiver_pubkey,
                        increment: increment_seconds,
                        amount: amount_usdc,
                        whirlpool,
                        tip: tip_usdc,
                        a_to_b: true,
                    },
                );
                match ix {
                    Ok(ix) => {
                        let mut tx = Transaction::new_with_payer(&[ix], Some(&pubkey));
                        let latest_blockhash = match rpc.get_latest_blockhash().await {
                            Ok(blockhash) => blockhash,
                            Err(err) => {
                                dioxus_logger::tracing::error!("Error getting latest blockhash: {:?}", err);
                                return None;
                            }
                        };
                        tx.message.recent_blockhash = latest_blockhash;
                        Some(tx)
                    }
                    Err(err) => {
                        dioxus_logger::tracing::error!("Error building ix: {:?}", err);
                        None
                    }
                }
            }
        }
    });

    rsx! {
        div {
            class: "paycheck-card",
        div {
            label { "Select number of days for recurring trade & send: " }
            select {
                onchange: move |e| {
                    let value = e.value().parse::<u64>().unwrap_or(1) as UnixTimestamp;
                    *selected_days.write() = value;
                },
                option { value: "0", "0 days" }
                option { value: "1", "1 day" }
                option { value: "7", "1 week" }
                option { value: "30", "1 month" }
            }
        }
        div {
                class: "text-input-container",
                label { "Receiver: " }
                input {
                    r#type: "text",
                    value: "{receiver}",
                    oninput: move |e| {
                        *receiver.write() = e.value().clone().to_string();
                    }
                }
        }
        div {
                class: "number-input-container",
                label { "Amount: " }
                input {
                    r#type: "number",
                    value: "{amount}",
                    oninput: move |e| {
                        let value = e.value().parse::<f64>().unwrap_or(0.0);
                        *amount.write() = value;
                    }
                }
                span { " USDC" }
            }
            div {
                class: "number-input-container",
                label { "Tip: " }
                input {
                    r#type: "number",
                    value: "{tip}",
                    oninput: move |e| {
                       if e.value().is_empty() {
                       *tip.write() = 0.0;
                        } else {
                        let value = e.value().parse::<f64>().unwrap_or(0.0);
                        *tip.write() = value;
                        }
                    }
                }
                span { " USDC" }
            }
        if let Some(Some(tx)) = tx.cloned() {
            match *status.read() {
                InvokeSignatureStatus::Start => rsx! {
                    button {
                        onclick: move |_| {
                            invoke_signature(tx.clone(), status);
                        },
                        "Submit memo transaction"
                    }
                },
                InvokeSignatureStatus::Waiting => rsx! { p { "Submitting..." } },
                InvokeSignatureStatus::DoneWithError => rsx! { p { "Error" } },
                InvokeSignatureStatus::Timeout => rsx! { p { "Timeout" } },
                InvokeSignatureStatus::Done(sig) => rsx! { p { "{sig}" } },
            }
        } else {
            p {
                "Loading tx"
            }
        }
            }
    }
}
