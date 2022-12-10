use crate::component::RegisterName;
use crate::interpreter::decode::{decode, Instruction};
use crate::interpreter::error::{ArithmeticOverflowSnafu, InterpreterError};
use crate::memory::Memory;

pub struct Interpreter {
    // reg[0] is pc
    reg: [u32; 32],
    mem: Box<dyn Memory>,
}

impl Interpreter {
    pub fn new(mem: Box<dyn Memory>) -> Self {
        let mut reg = [0; 32];
        reg[0] = 0x00400000; // pc
        reg[28] = 0x10008000; // gp
        reg[29] = 0x7ffffe40; // sp

        Interpreter { reg, mem }
    }

    fn pc(&self) -> u32 {
        self.reg[0]
    }

    fn reg(&self, reg: RegisterName) -> u32 {
        if reg.num() != 0 {
            self.reg[reg.num() as usize]
        } else {
            0
        }
    }

    fn set_pc(&mut self, val: u32) {
        self.reg[0] = val;
    }

    fn set_reg(&mut self, reg: RegisterName, val: u32) {
        if reg.num() != 0 {
            self.reg[reg.num() as usize] = val;
        }
    }

    fn step(&mut self) -> Result<(), InterpreterError> {
        let ins = decode(self.mem.read_u32(self.pc()))?;

        // `execute` is in its own separate module to prevent growing this file too big
        // see execute.rs in this directory
        self.execute(ins)
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), InterpreterError> {
        use Instruction::*;

        let mut pc = self.pc() + 4;

        match ins {
            add(x) => match i32::checked_add(self.reg(x.rs) as i32, self.reg(x.rt) as i32) {
                Some(val) => self.set_reg(x.rd, val as u32),
                None => ArithmeticOverflowSnafu {}.fail()?,
            },
            and(x) => {
                let val = self.reg(x.rs) & self.reg(x.rt);
                self.set_reg(x.rd, val);
            }
            or(x) => {
                let val = self.reg(x.rs) | self.reg(x.rt);
                self.set_reg(x.rd, val);
            }
            sub(x) => match i32::checked_sub(self.reg(x.rs) as i32, self.reg(x.rt) as i32) {
                Some(val) => self.set_reg(x.rd, val as u32),
                None => ArithmeticOverflowSnafu {}.fail()?,
            },
            slt(x) => {
                let cond = (self.reg(x.rs) as i32) < (self.reg(x.rt) as i32);
                self.set_reg(x.rd, cond.into());
            }
            lw(x) => {
                let sign_ext_imm = x.imm as i16 as i32 as u32;
                let addr = u32::wrapping_add(self.reg(x.rs), sign_ext_imm);
                self.set_reg(x.rt, self.mem.read_u32(addr));
            }
            sw(x) => {
                let sign_ext_imm = x.imm as i16 as i32 as u32;
                let addr = u32::wrapping_add(self.reg(x.rs), sign_ext_imm);
                self.mem.write_u32(addr, self.reg(x.rt));
            }
            beq(x) => {
                let sign_ext_imm = x.imm as i16 as i32 as u32;
                let branch_offset = sign_ext_imm << 2;

                if self.reg(x.rs) == self.reg(x.rt) {
                    pc = u32::wrapping_add(pc, branch_offset);
                }
            }
            j(x) => {
                let addr = (pc & 0xf000_0000) | ((x.addr & 0x3ff_ffff) << 2);
                pc = addr;
            }
        }

        self.set_pc(pc);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assembler::{assemble};
    use crate::memory::{create_memory, EndianMode};

    const TEXT_ADDR: u32 = 0x00400000;

    fn init_state(asm: &str) -> Interpreter {
        let segments = assemble(EndianMode::native(), asm).unwrap();
        let mem = create_memory(EndianMode::native(), &segments);
        Interpreter::new(mem)
    }

    #[test]
    fn create() {
        let mut mem = create_memory(EndianMode::native(), &[]);
        mem.write_u32(TEXT_ADDR, 0x00000020);
        Interpreter::new(mem);
    }

    #[test]
    fn add() {
        let mut state = init_state(".text\nadd $18, $16, $17\nadd $18, $19, $20");
        state.reg[16] = 1;
        state.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.reg[18], 3);

        state.reg[19] = i32::MAX as u32;
        state.reg[20] = 1;
        state.step().unwrap_err(); //TODO: Should I check if it is InterpreterError::ArithmeticOverflow?
        assert_eq!(state.reg[18], 3);
    }

    #[test]
    fn sub() {
        let mut state =
            init_state(".text\nsub $18, $16, $17\nsub $18, $16, $17\nsub $18, $16, $17");
        state.reg[16] = 3;
        state.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.reg[18], 1);

        state.reg[16] = 1;
        state.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.reg[18], -1_i32 as u32);

        state.reg[16] = i32::MIN as u32;
        state.reg[17] = 1;
        state.step().unwrap_err(); //TODO: Should I check if it is InterpreterError::ArithmeticOverflow?
        assert_eq!(state.reg[18], -1_i32 as u32);
    }

    #[test]
    fn and_or() {
        let mut state = init_state(".text\nand $18, $16, $17\nor $18, $16, $17");
        state.reg[16] = 13;
        state.reg[17] = 9;
        state.step().unwrap();
        assert_eq!(state.reg[18], 13 & 9);
        state.step().unwrap();
        assert_eq!(state.reg[18], 13 | 9);
    }

    #[test]
    fn slt() {
        let mut state = init_state(
            ".text\nslt $18, $16, $17\nslt $18, $16, $17\nslt $18, $16, $17\nslt $18, $16, $17",
        );
        state.reg[16] = 1;
        state.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.reg[18], 1);

        state.reg[16] = 2;
        state.reg[17] = 1;
        state.step().unwrap();
        assert_eq!(state.reg[18], 0);

        state.reg[16] = 1;
        state.reg[17] = -1_i32 as u32;
        state.step().unwrap();
        assert_eq!(state.reg[18], 0);

        state.reg[16] = -2_i32 as u32;
        state.reg[17] = -1_i32 as u32;
        state.step().unwrap();
        assert_eq!(state.reg[18], 1);
    }

    #[test]
    fn mem() {
        let mut state = init_state(".data 0x10008000\n.word -1234, 1234\n.text\nlw $16, 0($gp)\nlw $16, 4($gp)\nadd $16, $16, $17\nsw $16, 8($gp)");
        assert_eq!(state.mem.read_u32(0x10008004), 1234);
        state.reg[17] = 1;

        state.step().unwrap();
        assert_eq!(state.reg[16], -1234_i32 as u32);

        state.step().unwrap();
        assert_eq!(state.reg[16], 1234);

        state.step().unwrap();
        assert_eq!(state.reg[16], 1235);

        state.step().unwrap();
        assert_eq!(state.mem.read_u32(0x10008008), 1235);
    }

    #[test]
    fn jump() {
        let mut state = init_state(".text\nj 0x00001234");
        state.step().unwrap();
        assert_eq!(state.pc(), 0x1234);
    }

    #[test]
    fn beq() {
        let mut state = init_state(".text\nstart:\nadd $16, $16, $17\nbeq $16, $0, fin\nj start\nfin:\nj 0x00001234");
        state.reg[16] = 3;
        state.reg[17] = -1_i32 as u32;

        let expected_pc = [0, 4, 8, 0, 4, 8, 0, 4, 12];

        for pc in expected_pc {
            assert_eq!(pc + TEXT_ADDR, state.pc());
            state.step().unwrap();
        }
        assert_eq!(state.pc(), 0x1234);
    }
}
