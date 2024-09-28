use solana_program::decode_error::DecodeError;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Bonus Prize program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PaycheckProgramError {
    #[error("InvalidInstruction")]
    InvalidInstructionData,
    #[error("IntervalNotPassed")]
    IntervalNotPassed,
}
impl From<PaycheckProgramError> for ProgramError {
    fn from(e: PaycheckProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for PaycheckProgramError {
    fn type_of() -> &'static str {
        "BonusPrizeError"
    }
}
impl PrintProgramError for PaycheckProgramError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            PaycheckProgramError::InvalidInstructionData => msg!("Error: Invalid instruction"),
            PaycheckProgramError::IntervalNotPassed => {
                msg!("Error: Must wait for interval before executing again")
            }
        }
    }
}
