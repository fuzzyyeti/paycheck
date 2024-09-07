use solana_program::decode_error::DecodeError;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Bonus Prize program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PaycheckProgramError {
    #[error("ClaimerNotWinner")]
    ClaimerNotWinner,
    #[error("DrawNumberMismatch")]
    DrawNumberMismatch,
    #[error("DrawResultAccountDerivationError")]
    DrawResultAccountDerivationError,
    #[error("DrawResultAccountOwnerMismatch")]
    DrawResultAccountOwnerMismatch,
    #[error("InvalidBonusPrizeSigner")]
    InvalidBonusPrizeSigner,
    #[error("DrawResultDisciminatorMismatch")]
    DrawResultDiscriminatorMismatch,
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
            PaycheckProgramError::ClaimerNotWinner => msg!("Error: Claimer is not the winner"),
            PaycheckProgramError::DrawNumberMismatch => msg!("Error: Draw number mismatch"),
            PaycheckProgramError::DrawResultAccountDerivationError => {
                msg!("Error: Draw result account derivation error")
            }
            PaycheckProgramError::DrawResultAccountOwnerMismatch => {
                msg!("Error: Draw result account owner mismatch")
            }
            PaycheckProgramError::InvalidBonusPrizeSigner => msg!("Error: Invalid bonus prize signer"),
            PaycheckProgramError::DrawResultDiscriminatorMismatch => {
                msg!("Error: Draw result discriminator mismatch")
            }
        }
    }
}