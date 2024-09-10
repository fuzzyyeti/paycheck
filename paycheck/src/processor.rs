use crate::instructions::create_paycheck::{process_create_paycheck, CreatePaycheckArgs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PaycheckInstructions {
    CreateConfig(CreatePaycheckArgs),
    UpdateConfig,
    CreatePaycheck,
    EditPaycheck,
    ClosePaycheck,
    ActivatePaycheck,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = PaycheckInstructions::try_from_slice(instruction_data);
    match instruction {
        Ok(PaycheckInstructions::CreateConfig(config_args)) => {
            msg!("Got in here, {:?}", config_args);
            process_create_paycheck(program_id, accounts, config_args)?;
        }
        Ok(PaycheckInstructions::UpdateConfig) => {
            msg!("UpdateConfig");
        }
        Ok(PaycheckInstructions::CreatePaycheck) => {
            msg!("CreatePaycheck");
        }
        Ok(PaycheckInstructions::EditPaycheck) => {
            msg!("EditPaycheck");
        }
        Ok(PaycheckInstructions::ClosePaycheck) => {
            msg!("ClosePaycheck");
        }
        Ok(PaycheckInstructions::ActivatePaycheck) => {
            msg!("ActivatePaycheck");
        }
        Err(e) => {
            msg!("Error: {:?}", e);
        }
    }
    Ok(())
}
