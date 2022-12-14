use crate::executor::error::ExecuteError;
use crate::executor::Arch;
use crate::memory::Memory;

#[derive(Debug)]
pub struct DummyJit;

impl DummyJit {
    pub fn new(_mem: Box<dyn Memory>) -> Self {
        panic!("this platform does not support JIT");
    }

    pub fn as_arch(&self) -> &Arch {
        panic!("this platform does not support JIT");
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        panic!("this platform does not support JIT");
    }

    pub fn into_arch(self) -> Arch {
        panic!("this platform does not support JIT");
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        panic!("this platform does not support JIT");
    }

    pub fn exec(&mut self) -> Result<(), ExecuteError> {
        panic!("this platform does not support JIT");
    }

    pub fn invalidate(&mut self) {
        panic!("this platform does not support JIT");
    }
}
