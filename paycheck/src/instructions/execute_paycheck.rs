use crate::error::PaycheckProgramError;
use crate::processor::PaycheckInstructions;
use crate::ID;
use solana_program::account_info::{ AccountInfo};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use crate::consts::PAYCHECK_SEED;

pub fn process_execute_paycheck(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    Ok(())
}

pub fn execute_paycheck_ix(
    payer: Pubkey,
    creator: Pubkey,
    whirlpool: Pubkey,
    mint: Pubkey,
) -> Result<Instruction, PaycheckProgramError> {

    let paycheck= Pubkey::find_program_address(
        &[
            PAYCHECK_SEED,
            &whirlpool.to_bytes(),
            &creator.to_bytes(),
        ],
        &ID,
    )
        .0;

    let data = borsh::to_vec(&PaycheckInstructions::ExecutePaycheck)
        .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;

    let creator_ata = spl_associated_token_account::get_associated_token_address(
        &creator,
        &mint,
    );

    let paycheck_ata= spl_associated_token_account::get_associated_token_address(
        &paycheck,
        &mint,
    );

    Ok(Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(creator, false),
            AccountMeta::new(paycheck, false),
            AccountMeta::new(creator_ata, false),
            AccountMeta::new(paycheck_ata, false),
            AccountMeta::new(whirlpool, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(whirlpools_state::ID, false),
        ],
        data,
    })
}
