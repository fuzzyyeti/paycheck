#![allow(non_snake_case)]

use crate::hooks::use_wallet_adapter::{
    invoke_signature, use_wallet_adapter, use_wallet_adapter_provider, InvokeSignatureStatus,
    WalletAdapter,
};
use borsh::BorshDeserialize;
use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use paycheck::paycheck_seeds;
use paycheck::state::Paycheck;
use solana_client_wasm::{
    solana_sdk::{pubkey::Pubkey, transaction::Transaction},
    WasmClient,
};
use solana_extra_wasm::program::spl_associated_token_account::get_associated_token_address;
use solana_extra_wasm::program::spl_token;
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::pubkey;
use std::str::FromStr;
use tracing::Level;

mod hooks;

pub const RPC_URL: &str = "http://127.0.0.1:8899";

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[derive(PartialEq, Props, Clone)]
struct ShowPaycheckProps {
    paycheck: Paycheck,
    children: Element,
}

fn App() -> Element {
    use_wallet_adapter_provider();
    let wallet_adapter = use_wallet_adapter();
    let info = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let (paycheck_address, _) = Pubkey::find_program_address(
                    paycheck_seeds!(whirlpool, pubkey, true),
                    &paycheck::ID,
                );

                let client = WasmClient::new(RPC_URL);
                let paycheck_account = client.get_account(&paycheck_address).await;

                match paycheck_account {
                    Ok(account) => {
                        let data: Paycheck = Paycheck::try_from_slice(&account.data).unwrap();
                        Some(data)
                    }
                    Err(_) => None,
                }
            }
        }
    });

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
                match info.read().as_ref() {
                    Some(Some(paycheck)) => rsx! {
                        ShowPaychecks { paycheck: paycheck.clone() }  }
                    ,
                    _ => rsx! {
                        CreatePaycheck {}
                    }
                }
            }
            div {
                class: "bottom-button",
                a {
                    href: "https://twitter.com",
                    target: "_blank",
                    class: "twitter-button",
                    i { class: "fab fa-x-twitter" } // Update the icon class if Font Awesome has an "X" icon
                }
            }
            div {
                class: "bottom-text",
                p { "© fuzzy yeti 2024" }
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

fn ShowPaychecks(props: ShowPaycheckProps) -> Element {
    let paycheck = props.paycheck;
    let status = use_signal(|| InvokeSignatureStatus::Start);
    let wallet_adapter = use_wallet_adapter();

    let close_paycheck_tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let (paycheck_address, _) = Pubkey::find_program_address(
                    paycheck_seeds!(whirlpool, pubkey, true),
                    &paycheck::ID,
                );

                let client = WasmClient::new(RPC_URL);
                let paycheck_account = client.get_account(&paycheck_address).await;
                let result = match paycheck_account {
                    Ok(_) => {
                        let ix = paycheck::instructions::close_paycheck::create_close_paycheck_ix(
                            pubkey,
                            paycheck_address,
                        );
                        let mut tx = Transaction::new_with_payer(&[ix], Some(&pubkey));
                        let latest_blockhash = match client.get_latest_blockhash().await {
                            Ok(blockhash) => blockhash,
                            Err(err) => {
                                dioxus_logger::tracing::error!(
                                    "Error getting latest blockhash: {:?}",
                                    err
                                );
                                return None;
                            }
                        };
                        tx.message.recent_blockhash = latest_blockhash;
                        Some(tx)
                    }
                    Err(_) => None,
                };
                dioxus_logger::tracing::info!("close paycheck tx: {:?}", result);

                result
            }
        }
    });
    let increment_days = paycheck.increment / (24 * 60 * 60);

    let last_executed_date = match Utc.timestamp_opt(paycheck.last_executed, 0) {
        chrono::LocalResult::Single(datetime) => datetime.format("%m-%d-%y").to_string(),
        _ => "Invalid date".to_string(),
    };

    let ui_amount = paycheck.amount as f64 / 1_000_000.0;

    let ui_tip = (paycheck.tip as f64) / 1_000_000.0;

    rsx! {
        div {
            class: "paycheck-card",
            h2 { "Paycheck Active" }
            p { "Converting your ORE to USDC and sending to"}
            p { "{paycheck.receiver}" }
            p {
                style: "margin-top: 3rem;",
                "Increment: {increment_days} days" }
            p { "Last Executed: {last_executed_date}" }
            p { "Amount: {ui_amount} USDC" }
            p { "Tip: {ui_tip} USDC" }
            if let Some(Some(close_paycheck_tx)) = close_paycheck_tx.cloned() {
                match *status.read() {
                    InvokeSignatureStatus::Start => rsx! {
                        div {
                            class: "trash-icon",
                            onclick: move |_| {
                                invoke_signature(close_paycheck_tx.clone(), status);
                            },
                            i { class: "fas fa-trash" }
                        }
                    },
                    InvokeSignatureStatus::Waiting => rsx! { p { "Submitting..." } },
                    InvokeSignatureStatus::DoneWithError => rsx! { p { "Error" } },
                    InvokeSignatureStatus::Timeout => rsx! { p { "Timeout" } },
                    InvokeSignatureStatus::Done(_) => {
                    let _ = eval("window.location.reload();");
                        rsx! { p { "Reloading..." } }
                    },
                }
            }
        }
    }
}

fn CreatePaycheck() -> Element {
    let status = use_signal(|| InvokeSignatureStatus::Start);
    let wallet_adapter = use_wallet_adapter();
    let mut selected_days = use_signal(|| 0); // Default to 1 day
    let mut amount = use_signal(|| 5.0); // Default amount
    let mut tip = use_signal(|| 0.10); //
    let mut receiver = use_signal(String::new);

    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let rpc = WasmClient::new(RPC_URL);
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let bsol = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
                tracing::info!("selected_days: {:?}", selected_days);
                let increment_seconds = selected_days * 24 * 60 * 60; // Convert days to seconds
                let amount_usdc = (amount * 1_000_000.0) as u64;
                let tip_usdc = (tip * 1_000_000.0) as u64;

                let receiver_pubkey = Pubkey::from_str(&receiver.read()).unwrap_or(pubkey);
                let source_wallet = get_associated_token_address(&pubkey, &bsol);
                let paycheck = Pubkey::find_program_address(
                    paycheck_seeds!(whirlpool, pubkey, true),
                    &paycheck::ID,
                )
                .0;

                let delegate_ix = match spl_token::instruction::approve(
                    &spl_token::ID,
                    &source_wallet,
                    &paycheck,
                    &pubkey,
                    &[],
                    u64::MAX,
                ) {
                    Ok(ix) => ix,
                    Err(err) => {
                        dioxus_logger::tracing::error!("Error building ix: {:?}", err);
                        return None;
                    }
                };

                let ix = match paycheck::instructions::create_paycheck::create_paycheck_ix(
                    pubkey,
                    paycheck::instructions::create_paycheck::CreatePaycheckArgs {
                        receiver: receiver_pubkey,
                        increment: increment_seconds,
                        amount: amount_usdc,
                        whirlpool,
                        tip: tip_usdc,
                        a_to_b: true,
                    },
                ) {
                    Ok(ix) => ix,
                    Err(err) => {
                        dioxus_logger::tracing::error!("Error building ix: {:?}", err);
                        return None;
                    }
                };
                let mut tx = Transaction::new_with_payer(&[delegate_ix, ix], Some(&pubkey));
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
        }
    });

    rsx! {
        div {
            class: "paycheck-card",
        div {
            class: "input-container",
            label {
                    class: "interval-label",
                    "Trade & send interval: "
                }
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
                class: "input-container",
                label {
                    class: "input-label",
                    "Receiver: "
                }
                input {
                    r#type: "text",
                    value: "{receiver}",
                    oninput: move |e| {
                        *receiver.write() = e.value().clone().to_string();
                    }
                }
        }
        div {
                class: "input-container",
                label {
                    class: "input-label",
                    "Amount: "
                }
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
                class: "input-container",
                label {
                    class: "input-label",
                    "Tip: " }
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
                        "Create Paycheck"
                    }
                },
                InvokeSignatureStatus::Waiting => rsx! { p { "Submitting..." } },
                InvokeSignatureStatus::DoneWithError => rsx! { p { "Error" } },
                InvokeSignatureStatus::Timeout => rsx! { p { "Timeout" } },
                InvokeSignatureStatus::Done(_) => {
                    let _ = eval("window.location.reload();");
                        rsx! { p { "Reloading..." } }
                },
            }
        } else {
            p {
                "Connect Your Wallet"
            }
        }
        }
    }
}
