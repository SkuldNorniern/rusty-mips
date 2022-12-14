use crate::executor::error::ExecuteError;
use crate::executor::interpreter::Interpreter;
use crate::executor::jit::Jit;
use crate::executor::pipeline::processor::Pipeline;
use crate::executor::Arch;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Executor {
    ExInterpreter(Interpreter),
    ExJit(Jit),
    ExPipeline(Pipeline),
}

impl Executor {
    pub fn as_arch(&self) -> &Arch {
        match self {
            Executor::ExInterpreter(x) => x.as_arch(),
            Executor::ExJit(x) => x.as_arch(),
            Executor::ExPipeline(x) => x.as_arch(),
        }
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        match self {
            Executor::ExInterpreter(x) => x.as_arch_mut(),
            Executor::ExJit(x) => x.as_arch_mut(),
            Executor::ExPipeline(x) => x.as_arch_mut(),
        }
    }

    pub fn into_arch(self) -> Arch {
        match self {
            Executor::ExInterpreter(x) => x.into_arch(),
            Executor::ExJit(x) => x.into_arch(),
            Executor::ExPipeline(x) => x.into_arch(),
        }
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        match self {
            Executor::ExInterpreter(x) => x.step(),
            Executor::ExJit(x) => x.step(),
            Executor::ExPipeline(x) => {
                x.step();
                Ok(())
            }
        }
    }

    pub fn exec(&mut self) -> Result<(), ExecuteError> {
        match self {
            Executor::ExInterpreter(x) => x.step(),
            Executor::ExJit(x) => x.exec(),
            Executor::ExPipeline(x) => {
                x.step();
                Ok(())
            }
        }
    }
}
