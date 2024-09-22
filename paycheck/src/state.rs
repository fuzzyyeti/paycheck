use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Paycheck {
    pub creator: Pubkey,
    pub receiver: Pubkey,
    pub start_date: u64,
    pub increment: u64,
    pub amount: u64,
    pub whirlpool: Pubkey,
    pub target_mint: Pubkey,
    pub tip: u64,
    pub is_enabled: bool,
    pub bump: u8,
}

impl Paycheck {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 32 + 32 + 8 + 1 + 1;
}
