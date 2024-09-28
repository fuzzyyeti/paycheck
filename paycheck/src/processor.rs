use crate::instructions::create_paycheck::{process_create_paycheck, CreatePaycheckArgs};
use crate::instructions::{process_close_paycheck, process_execute_paycheck, ExecutePaycheckArgs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PaycheckInstructions {
    CreatePaycheck(CreatePaycheckArgs),
    ExecutePaycheck(ExecutePaycheckArgs),
    ClosePaycheck(),
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = PaycheckInstructions::try_from_slice(instruction_data);
    match instruction {
        Ok(PaycheckInstructions::CreatePaycheck(create_paycheck_args)) => {
            process_create_paycheck(program_id, accounts, create_paycheck_args)?;
        }
        Ok(PaycheckInstructions::ExecutePaycheck(execute_paycheck_args)) => {
            process_execute_paycheck(program_id, accounts, execute_paycheck_args)?;
        }
        Ok(PaycheckInstructions::ClosePaycheck()) => {
            process_close_paycheck(program_id, accounts)?;
        }
        Err(e) => {
            msg!("Error: {:?}", e);
        }
    }
    Ok(())
}
