use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{msg, system_instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use mpl_macros::{assert_derivation, assert_signer};
use crate::error::PaycheckProgramError;
use crate::ID;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;

const PAYCHECK_SEED: &[u8] = b"paycheck";

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone)]
pub struct CreatePaycheckArgs {
    pub receiver: Pubkey,
    pub start_date: u64,
    pub increment: u64,
    pub amount: u64,
    pub whirlpool: Pubkey,
}


pub fn process_create_paycheck(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    config_args: CreatePaycheckArgs
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.into_iter();
    let creator = next_account_info(account_info_iter)?;
    let paycheck_account = next_account_info(account_info_iter)?;
    assert_signer(creator)?;
    let bump = assert_derivation(&program_id, &paycheck_account,
   &[PAYCHECK_SEED,
       &config_args.whirlpool.to_bytes(),
       &creator.key.to_bytes()],
                      ProgramError::InvalidSeeds)?;
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(Paycheck::LEN);
    invoke_signed(
        &system_instruction::create_account(
            &creator.key,
            &paycheck_account.key,
            required_lamports,
            Paycheck::LEN as u64,
            program_id,
        ),
        &[
            creator.clone(),
            paycheck_account.clone(),
        ],
        &[&[PAYCHECK_SEED,
            &config_args.whirlpool.to_bytes(),
            &creator.key.to_bytes(),
            &[bump]]],
    )?;
    let mut config_account = paycheck_account.try_borrow_mut_data()?;
    let mut paycheck_account_data = Paycheck::try_from_slice(&config_account)?;
    paycheck_account_data.receiver = config_args.receiver;
    paycheck_account_data.start_date = config_args.start_date;
    paycheck_account_data.increment = config_args.increment;
    paycheck_account_data.amount = config_args.amount;
    paycheck_account_data.whirlpool = config_args.whirlpool;
    paycheck_account_data.is_enabled = true;
    paycheck_account_data.bump = bump;

    // Save the updated data back to the account
    paycheck_account_data.serialize(&mut *config_account)?;

    Ok(())
}

pub fn create_paycheck_ix(
    creator: Pubkey,
    config_args: CreatePaycheckArgs
) -> Result<Instruction, PaycheckProgramError> {
    let data =
       borsh::to_vec(&PaycheckInstructions::CreateConfig(config_args.clone()))
       .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;
    let paycheck_account = Pubkey::find_program_address(
        &[PAYCHECK_SEED,
            &config_args.whirlpool.to_bytes(),
            &creator.to_bytes()],
        &ID,
    ).0;

    Ok(Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(paycheck_account, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),],
        data
    })
}