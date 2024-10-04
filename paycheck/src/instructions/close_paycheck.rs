use crate::paycheck_seeds_with_bump;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;
use borsh::BorshDeserialize;
use mpl_macros::{assert_derivation_with_bump, assert_signer};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

pub fn process_close_paycheck(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let creator = next_account_info(account_info_iter)?;
    let paycheck = next_account_info(account_info_iter)?;
    let paycheck_data: Paycheck = Paycheck::try_from_slice(&paycheck.data.borrow())?;
    assert_derivation_with_bump(
        program_id,
        paycheck,
        paycheck_seeds_with_bump!(
            paycheck_data.whirlpool,
            creator.key,
            paycheck_data.a_to_b,
            paycheck_data.bump
        ),
        ProgramError::InvalidSeeds,
    )?;
    assert_signer(creator)?;
    let paycheck_data = Paycheck::try_from_slice(&paycheck.data.borrow())?;
    assert_eq!(paycheck_data.creator, *creator.key);

    //move all lamports to creator
    **creator.try_borrow_mut_lamports()? += paycheck.lamports();
    **paycheck.try_borrow_mut_lamports()? = 0;

    paycheck.realloc(0, false)?;
    paycheck.assign(&system_program::id());

    Ok(())
}

pub fn create_close_paycheck_ix(creator: Pubkey, paycheck: Pubkey) -> Instruction {
    let data = borsh::to_vec(&PaycheckInstructions::ClosePaycheck()).unwrap();
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(paycheck, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data,
    }
}
