use crate::component::RegisterName;
use crate::memory::Memory;

#[repr(C)]
#[derive(Debug)]
pub struct Arch {
    // reg[32] is pc
    pub(super) reg: [u32; 33],

    // below here is inaccessible from JIT. May use Rust-specific types.
    pub(super) mem: Box<dyn Memory>,
}

impl Arch {
    pub fn mem(&self) -> &dyn Memory {
        &*self.mem
    }

    pub fn mem_mut(&mut self) -> &mut dyn Memory {
        &mut *self.mem
    }

    pub fn read_all_reg(&self, dst: &mut [u32]) {
        dst.copy_from_slice(&self.reg[..32]);
    }

    pub fn pc(&self) -> u32 {
        self.reg[32]
    }

    pub fn reg(&self, reg: RegisterName) -> u32 {
        self.reg[reg.num() as usize]
    }

    pub fn set_pc(&mut self, val: u32) {
        self.reg[32] = val;
    }

    pub fn set_reg(&mut self, reg: RegisterName, val: u32) {
        if reg.num() != 0 {
            self.reg[reg.num() as usize] = val;
        }
    }
}
