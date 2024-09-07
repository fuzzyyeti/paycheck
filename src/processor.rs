use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::pubkey::Pubkey;
use crate::error::PaycheckProgramError;

#[repr(u8)]
#[derive(Clone, Debug, Copy, PartialEq, Zeroable)]
pub enum PaycheckInstructions {
    CreateConfig,
    UpdateConfig,
    CreatePaycheck,
    EditPaycheck,
    ClosePaycheck,
    ActivatePaycheck,
}

unsafe impl Pod for PaycheckInstructions {}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("instruction_data: {:?}", instruction_data);
    let (discriminant, rest) = instruction_data.split_first().ok_or(PaycheckProgramError::ClaimerNotWinner)?;
    let discriminant = [*discriminant; 1];
    match bytemuck::from_bytes::<PaycheckInstructions>(&discriminant) {
        PaycheckInstructions::CreateConfig => {
            msg!("CreateConfig");
        }
        PaycheckInstructions::UpdateConfig => {
            msg!("UpdateConfig");
        }
        PaycheckInstructions::CreatePaycheck => {
            msg!("CreatePaycheck");
        }
        PaycheckInstructions::EditPaycheck => {
            msg!("EditPaycheck");
        }
        PaycheckInstructions::ClosePaycheck => {
            msg!("ClosePaycheck");
        }
        PaycheckInstructions::ActivatePaycheck => {
            msg!("ActivatePaycheck");
        }
    }

    Ok(())
}