use solana_program::declare_id;

pub mod consts;
mod entrypoint;
mod error;
pub mod instructions;
pub mod processor;
pub mod state;
mod utils;

declare_id!("5FYHsXPR2hvoXJzMe8GBhFrfggRfcgKC6mxCFo9dGMdo");
