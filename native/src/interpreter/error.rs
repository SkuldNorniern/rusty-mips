use snafu::prelude::*;
use snafu::Backtrace;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum InterpreterError {
    #[snafu(display("unknown opcode 0x{opcode:02x}"))]
    UnknownOpcode { opcode: u8, backtrace: Backtrace },

    #[snafu(display("unknown funct field 0x{funct:02x}"))]
    UnknownFunct { funct: u8, backtrace: Backtrace },

    #[snafu(display("overflowed arithmetic operation"))]
    ArithmeticOverflow { backtrace: Backtrace },
}
