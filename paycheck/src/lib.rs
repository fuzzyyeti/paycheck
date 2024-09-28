use solana_program::declare_id;

pub mod consts;
mod entrypoint;
mod error;
pub mod instructions;
pub mod processor;
pub mod state;

declare_id!("54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4");
