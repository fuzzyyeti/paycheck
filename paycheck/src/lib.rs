use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

pub const ID: Pubkey = pubkey!("54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4");
mod entrypoint;
mod error;
pub mod instructions;
pub mod processor;
pub mod state;
