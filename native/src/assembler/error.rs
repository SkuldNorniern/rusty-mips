use std::ops::RangeInclusive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("unexpected trailing token `{0}`")]
    TrailingToken(String),

    #[error("invalid token `{0}`")]
    InvalidToken(String),

    #[error("required argument not found")]
    RequiredArgNotFound,

    #[error("segment declaration required for token `{0}`")]
    SegmentRequired(String),

    #[error("base address `{0}` out of expected range `{1:?}`")]
    BaseAddressOutOfRange(u32, RangeInclusive<u32>),

    #[error("unknown instruction `{0}")]
    UnknownInstruction(String),

    #[error("invalid register name `{0}")]
    InvalidRegisterName(String),

    #[error("invalid number of operands in line `{0}`")]
    InvalidNumberOfOperands(String),

    #[error("line too long")]
    LineTooLong,
}
