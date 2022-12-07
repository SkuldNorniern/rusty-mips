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

    #[error("segment address overlaps")]
    SegmentOverlap,

    #[error("offset {0} is too large to encode")]
    OffsetTooLarge(i64),

    #[error("branch offset {0} is unaligned")]
    BranchOffsetUnaligned(i64),

    #[error("jump target 0x{target:08x} is too far to encode from pc=0x{pc:08x}")]
    JumpTooFar { target: u32, pc: u32 },

    #[error("jump target {0} is unaligned")]
    JumpTargetUnaligned(u32),

    #[error("label `{0}` was not found")]
    LabelNotFound(String),
}
