use crate::consts::PAYCHECK_SEED;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;
use borsh::BorshDeserialize;
use mpl_macros::{assert_derivation_with_bump, assert_signer};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{msg, system_program};

pub fn process_close_paycheck(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let creator = next_account_info(account_info_iter)?;
    let paycheck = next_account_info(account_info_iter)?;
    let paycheck_data: Paycheck = Paycheck::try_from_slice(&paycheck.data.borrow())?;
    let system_program = next_account_info(account_info_iter)?;
    msg!("got here");
    assert_derivation_with_bump(
        program_id,
        paycheck,
        &[
            PAYCHECK_SEED,
            &paycheck_data.whirlpool.to_bytes(),
            &paycheck_data.creator.to_bytes(),
            &[paycheck_data.bump],
        ],
        ProgramError::InvalidSeeds,
    )?;
    msg!("got there");
    assert_signer(creator)?;
    msg!("creator: {:?}", creator.key);
    let paycheck_data = Paycheck::try_from_slice(&paycheck.data.borrow())?;
    assert_eq!(paycheck_data.creator, *creator.key);

    msg!("paycheck_data: {:?}", paycheck_data);
    //move all lamports to creator
    **creator.try_borrow_mut_lamports()? += paycheck.lamports();
    **paycheck.try_borrow_mut_lamports()? = 0;
    msg!("paycheck lamports: {:?}", paycheck.lamports());

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
