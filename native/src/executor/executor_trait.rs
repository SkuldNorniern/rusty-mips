use crate::executor::error::ExecuteError;
use crate::executor::interpreter::Interpreter;
use crate::executor::jit::Jit;
use crate::executor::Arch;

#[derive(Debug)]
pub enum Executor {
    ExInterpreter(Interpreter),
    ExJit(Jit),
}

impl Executor {
    pub fn as_arch(&self) -> &Arch {
        match self {
            Executor::ExInterpreter(x) => x.as_arch(),
            Executor::ExJit(x) => x.as_arch(),
        }
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        match self {
            Executor::ExInterpreter(x) => x.as_arch_mut(),
            Executor::ExJit(x) => x.as_arch_mut(),
        }
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        match self {
            Executor::ExInterpreter(x) => x.step(),
            Executor::ExJit(x) => x.step(),
        }
    }
}
