use crate::error::PaycheckProgramError;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;
use crate::{paycheck_seeds, paycheck_seeds_with_bump, ID};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_macros::{assert_derivation, assert_signer};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::clock::UnixTimestamp;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::{system_instruction};
use solana_program::sysvar::Sysvar;

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone)]
pub struct CreatePaycheckArgs {
    pub receiver: Pubkey,
    pub increment: UnixTimestamp,
    pub amount: u64,
    pub tip: u64,
    pub whirlpool: Pubkey,
    pub a_to_b: bool,
}

pub fn process_create_paycheck(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    config_args: CreatePaycheckArgs,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let creator = next_account_info(account_info_iter)?;
    let paycheck_account = next_account_info(account_info_iter)?;

    assert_signer(creator)?;

    let bump = assert_derivation(
        program_id,
        paycheck_account,
        paycheck_seeds!(
            config_args.whirlpool,
            creator.key,
            config_args.a_to_b
        ),
        ProgramError::InvalidSeeds,
    )?;
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(Paycheck::LEN);

    // Create the Paycheck account
    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            paycheck_account.key,
            required_lamports,
            Paycheck::LEN as u64,
            program_id,
        ),
        &[creator.clone(), paycheck_account.clone()],
        &[paycheck_seeds_with_bump!(
            config_args.whirlpool,
            creator.key,
            config_args.a_to_b,
            bump
            )],
    )?;

    // Initialize the Paycheck account data
    let mut config_account = paycheck_account.try_borrow_mut_data()?;
    let mut paycheck_account_data = Paycheck::try_from_slice(&config_account)?;
    paycheck_account_data.creator = *creator.key;
    paycheck_account_data.receiver = config_args.receiver;
    paycheck_account_data.increment = config_args.increment;
    paycheck_account_data.last_executed = 0;
    paycheck_account_data.amount = config_args.amount;
    paycheck_account_data.whirlpool = config_args.whirlpool;
    paycheck_account_data.tip = config_args.tip;
    paycheck_account_data.bump = bump;
    paycheck_account_data.a_to_b = config_args.a_to_b;

    // Save the updated data back to the account
    paycheck_account_data.serialize(&mut *config_account)?;

    Ok(())
}

pub fn create_paycheck_ix(
    creator: Pubkey,
    create_payckeck_args: CreatePaycheckArgs,
) -> Result<Instruction, PaycheckProgramError> {
    let data = borsh::to_vec(&PaycheckInstructions::CreatePaycheck(
        create_payckeck_args.clone(),
    ))
    .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;
    let paycheck_account = Pubkey::find_program_address(
        paycheck_seeds!(
            create_payckeck_args.whirlpool,
            creator,
            create_payckeck_args.a_to_b
        ),
        &ID,
    ).0;

    Ok(Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(paycheck_account, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data,
    })
}
