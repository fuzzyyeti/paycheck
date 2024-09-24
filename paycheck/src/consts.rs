use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

pub const PAYCHECK_SEED: &[u8] = b"paycheck";

pub const SWAP_DISCRIMINATOR: [u8; 8] = [ 0x2b, 0x04, 0xed, 0x0b, 0x1a, 0xc9, 0x1e, 0x62];

pub const MEMO_PROGRAM_ID: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
