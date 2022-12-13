use crate::executor::error::ExecuteError;
use crate::executor::{Arch, Interpreter};
use crate::memory::Memory;
use dynasmrt::{AssemblyOffset, ExecutableBuffer};
use std::collections::HashMap;
use std::mem;

type CompiledFunction = extern "win64" fn(&mut Arch);

#[derive(Debug)]
struct CompiledCode {
    offset: AssemblyOffset,
    buf: ExecutableBuffer,
}

#[derive(Debug)]
pub struct Jit {
    interpreter: Interpreter,
    codes: HashMap<u32, CompiledCode>,
}

impl Jit {
    pub fn new(mem: Box<dyn Memory>) -> Self {
        Jit {
            interpreter: Interpreter::new(mem),
            codes: HashMap::new(),
        }
    }

    pub fn as_arch(&self) -> &Arch {
        self.interpreter.as_arch()
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        self.interpreter.as_arch_mut()
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        let addr_from = self.interpreter.as_arch().pc();

        let code = match self.codes.get(&addr_from) {
            Some(x) => x,
            None => {
                match self.compile(addr_from) {
                    Ok(x) => x,
                    Err(_) => {
                        //FIXME: unwrap
                        return self.interpreter.step();
                    }
                }
            }
        };

        let f: CompiledFunction = unsafe { mem::transmute(code.buf.ptr(code.offset)) };
        f(self.interpreter.as_arch_mut());

        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.codes.clear();
    }

    fn compile(&mut self, _addr_from: u32) -> Result<&CompiledCode, ()> {
        Err(())
    }
}
