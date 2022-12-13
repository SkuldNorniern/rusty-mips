use snafu::prelude::*;
use snafu::Backtrace;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum ExecuteError {
    #[snafu(display("invalid or unsupported instruction 0x{ins:08x}"))]
    InvalidInstruction { ins: u32, backtrace: Backtrace },

    #[snafu(display("overflowed arithmetic operation"))]
    ArithmeticOverflow { backtrace: Backtrace },
}
