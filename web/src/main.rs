#![allow(non_snake_case)]

use borsh::BorshDeserialize;
use dioxus::prelude::*;
use hooks::use_wallet_adapter::use_balance;
use solana_client_wasm::{
    solana_sdk::{native_token::lamports_to_sol, transaction::Transaction, pubkey::Pubkey},
    WasmClient,
};
use solana_sdk::pubkey;
use solana_extra_wasm::program::spl_associated_token_account::get_associated_token_address;
use solana_extra_wasm::program::spl_associated_token_account::instruction::create_associated_token_account;
use solana_extra_wasm::program::spl_token;
use tracing::Level;
use paycheck::state::Paycheck;
use crate::hooks::use_wallet_adapter::{
    invoke_signature, use_wallet_adapter, use_wallet_adapter_provider, InvokeSignatureStatus,
    WalletAdapter,
};

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
        MountWalletAdapter {}
        RenderBalance {}
        SignMemo {}
        ShowPaychecks {}
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

fn RenderBalance() -> Element {
    let balance = use_balance();
    let e = match *balance.read() {
        Some(bal) => {
            rsx! {
                div {
                    "Balance: {lamports_to_sol(bal)} SOL"
                }
            }
        }
        None => {
            rsx! {
                div {
                    "Loading balance"
                }
            }
        }
    };
    e
}

fn ShowPaychecks() -> Element {
    let wallet_adapter = use_wallet_adapter();
    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                tracing::info!("connected the show paychecks {:?}", pubkey);
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let paycheck_address = Pubkey::find_program_address(
                    &[b"paycheck",
                        &whirlpool.to_bytes(),
                        &pubkey.to_bytes()],
                        &paycheck::id()).0;

                let client = WasmClient::new(RPC_URL);
                let paycheck_account = client.get_account(&paycheck_address).await;
                tracing::info!("paycheck account: {:?}", paycheck_account);
                let result = match paycheck_account {
                    Ok(account) => {
                        tracing::info!("paycheck account 2 : {:?}", account);
                        let data : Paycheck = Paycheck::try_from_slice(&account.data).unwrap();
                        Some(data)
                    }
                    Err(_) => None
                };
                dioxus_logger::tracing::info!("paycheck data: {:?}", result);

                Some("test")
            }
        }
    });
    rsx! {
        div {
            "Show paychecks 1"
        }
    }
}

fn SignMemo() -> Element {
    let status = use_signal(|| InvokeSignatureStatus::Start);
    let wallet_adapter = use_wallet_adapter();

    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let rpc = WasmClient::new(RPC_URL);
                let whirlpool = pubkey!("HGw4exa5vdxhJHNVyyxhCc6ZwycwHQEVzpRXMDPDAmVP");
                let bsol = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
                let receiver_wallet = pubkey!("2qztb9WG2sGGArmKitC7wwCDMwHTLSVSyqvQCQGY4da5");
                let receiver = get_associated_token_address(
                    &receiver_wallet,
                    &bsol);
                // let create_receiver_ix = create_associated_token_account(
                //     &pubkey,
                //     receiver_wallet,
                //     bsol,
                //     &spl_token::id(),
                // );
                let ix = paycheck::instructions::create_paycheck::create_paycheck_ix(
                    pubkey,
                    paycheck::instructions::create_paycheck::CreatePaycheckArgs {
                        receiver,
                        increment: 8,
                        amount: 1_000_000,
                        whirlpool,
                        tip: 10_000,
                        a_to_b: false,
                    },
                );
                match ix {
                    Ok(ix) => {
                        let mut tx = Transaction::new_with_payer(&[ix], Some(&pubkey));
                        tx.message.recent_blockhash = rpc.get_latest_blockhash().await.unwrap();
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
