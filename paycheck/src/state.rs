use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct Paycheck {
    pub creator: Pubkey,
    pub receiver: Pubkey,
    pub increment: i64,
    pub last_executed: UnixTimestamp,
    pub amount: u64,
    pub whirlpool: Pubkey,
    pub tip: u64,
    pub a_to_b: bool,
    pub bump: u8,
}

impl Paycheck {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 32 + 8 + 1 + 1;
}
