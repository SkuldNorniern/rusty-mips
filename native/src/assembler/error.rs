use snafu::prelude::*;
use snafu::Backtrace;
use std::ops::RangeInclusive;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum AssemblerError {
    #[snafu(display("unexpected trailing token `{token}` in line `{line}`"))]
    TrailingToken {
        token: String,
        line: String,
        backtrace: Backtrace,
    },

    #[snafu(display("expected number but got token `{token}`"))]
    TokenNotNumber { token: String, backtrace: Backtrace },

    #[snafu(display("expected register but got token `{token}`"))]
    TokenNotRegister { token: String, backtrace: Backtrace },

    #[snafu(display("invalid token `{token}`"))]
    InvalidToken { token: String, backtrace: Backtrace },

    #[snafu(display("segment declaration required for line `{line}`"))]
    SegmentRequired { line: String, backtrace: Backtrace },

    #[snafu(display("base address `{addr}` out of expected range `{range:?}`"))]
    BaseAddressOutOfRange {
        addr: u32,
        range: RangeInclusive<u32>,
        backtrace: Backtrace,
    },

    #[snafu(display("base address `{addr}` is too large to be linked"))]
    BaseAddressTooLarge { addr: u64, backtrace: Backtrace },

    #[snafu(display("unknown instruction `{ins}`"))]
    UnknownInstruction { ins: String, backtrace: Backtrace },

    #[snafu(display("invalid register name `{reg}`"))]
    InvalidRegisterName { reg: String, backtrace: Backtrace },

    #[snafu(display("invalid number of operands in line `{line}`"))]
    InvalidNumberOfOperands { line: String, backtrace: Backtrace },

    #[snafu(display("segment address overlaps"))]
    SegmentOverlap { backtrace: Backtrace },

    #[snafu(display("immediate {imm} is too large to encode"))]
    ImmediateTooLarge { imm: i64, backtrace: Backtrace },

    #[snafu(display("branch offset {offset} is unaligned"))]
    BranchOffsetUnaligned { offset: i64, backtrace: Backtrace },

    #[snafu(display("jump target 0x{target:08x} is too far to encode from pc=0x{pc:08x}"))]
    JumpTooFar {
        target: u32,
        pc: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("jump target 0x{target:08x} is unaligned"))]
    JumpTargetUnaligned { target: u32, backtrace: Backtrace },

    #[snafu(display("label `{label}` was not found"))]
    LabelNotFound { label: String, backtrace: Backtrace },
}
