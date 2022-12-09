use std::ops::RangeInclusive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("unknown opcode 0x{0:02x}")]
    UnknownOpcode(u8),

    #[error("unknown funct field 0x{0:02x}")]
    UnknownFunct(u8),

    #[error("overflowed arithmetic operation")]
    ArithmeticOverflow
}
