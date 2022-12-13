use crate::component::{Instruction, RegisterName, TypeI};
use crate::executor::error::*;
use crate::executor::Arch;
use crate::memory::Memory;

fn branch_offset(x: TypeI) -> u32 {
    (x.imm as i16 as i32 as u32) << 2
}

#[derive(Debug)]
pub struct Interpreter {
    arch: Arch,
}

impl Interpreter {
    pub fn as_arch(&self) -> &Arch {
        &self.arch
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        &mut self.arch
    }

    pub fn new(mem: Box<dyn Memory>) -> Self {
        let mut reg = [0; 33];
        reg[28] = 0x10008000; // gp
        reg[29] = 0x7ffffe40; // sp
        reg[32] = 0x00400024; // pc

        Interpreter {
            arch: Arch { reg, mem },
        }
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        let ins = Instruction::decode(self.arch.mem.read_u32(self.arch.pc()));

        if let Some(x) = ins.as_invalid() {
            return InvalidInstructionSnafu { ins: x }.fail();
        }

        self.execute(ins)
    }

    fn reg(&self, reg: RegisterName) -> u32 {
        self.arch.reg(reg)
    }

    fn set_reg(&mut self, reg: RegisterName, val: u32) {
        self.arch.set_reg(reg, val)
    }

    fn handle_syscall(&mut self) {
        // do nothing (for now)
    }

    fn execute(&mut self, ins: Instruction) -> Result<(), ExecuteError> {
        use Instruction::*;

        let mut pc = self.arch.pc() + 4;

        match ins {
            add(x) => match i32::checked_add(self.reg(x.rs) as i32, self.reg(x.rt) as i32) {
                Some(val) => self.set_reg(x.rd, val as u32),
                None => ArithmeticOverflowSnafu {}.fail()?,
            },
            addu(x) => {
                let val = u32::wrapping_add(self.reg(x.rs), self.reg(x.rt));
                self.set_reg(x.rd, val);
            }
            and(x) => {
                let val = self.reg(x.rs) & self.reg(x.rt);
                self.set_reg(x.rd, val);
            }
            nor(x) => {
                let val = !(self.reg(x.rs) | self.reg(x.rt));
                self.set_reg(x.rd, val);
            }
            or(x) => {
                let val = self.reg(x.rs) | self.reg(x.rt);
                self.set_reg(x.rd, val);
            }
            slt(x) => {
                let cond = (self.reg(x.rs) as i32) < (self.reg(x.rt) as i32);
                self.set_reg(x.rd, cond.into());
            }
            sltu(x) => {
                let cond = self.reg(x.rs) < self.reg(x.rt);
                self.set_reg(x.rd, cond.into());
            }
            sub(x) => match i32::checked_sub(self.reg(x.rs) as i32, self.reg(x.rt) as i32) {
                Some(val) => self.set_reg(x.rd, val as u32),
                None => ArithmeticOverflowSnafu {}.fail()?,
            },
            subu(x) => {
                let val = u32::wrapping_sub(self.reg(x.rs), self.reg(x.rt));
                self.set_reg(x.rd, val);
            }
            xor(x) => {
                let val = self.reg(x.rs) ^ self.reg(x.rt);
                self.set_reg(x.rd, val);
            }
            sll(x) => {
                let val = self.reg(x.rt) << x.shamt;
                self.set_reg(x.rd, val);
            }
            sllv(x) => {
                let val = self.reg(x.rt) << self.reg(x.rs);
                self.set_reg(x.rd, val);
            }
            sra(x) => {
                let val = (self.reg(x.rt) as i32) >> x.shamt;
                self.set_reg(x.rd, val as u32);
            }
            srav(x) => {
                let val = (self.reg(x.rt) as i32) >> self.reg(x.rs);
                self.set_reg(x.rd, val as u32);
            }
            srl(x) => {
                let val = self.reg(x.rt) >> x.shamt;
                self.set_reg(x.rd, val);
            }
            srlv(x) => {
                let val = self.reg(x.rt) >> self.reg(x.rs);
                self.set_reg(x.rd, val);
            }
            addi(x) => match i32::checked_add(self.reg(x.rs) as i32, x.imm as i16 as i32) {
                Some(val) => self.set_reg(x.rt, val as u32),
                None => ArithmeticOverflowSnafu {}.fail()?,
            },
            addiu(x) => {
                let val = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, val);
            }
            andi(x) => {
                let val = self.reg(x.rs) & (x.imm as u32);
                self.set_reg(x.rt, val);
            }
            lui(x) => {
                self.set_reg(x.rt, (x.imm as u32) << 16);
            }
            ori(x) => {
                let val = self.reg(x.rs) | (x.imm as u32);
                self.set_reg(x.rt, val);
            }
            slti(x) => {
                let val = ((self.reg(x.rs) as i32) < (x.imm as i16 as i32)).into();
                self.set_reg(x.rt, val);
            }
            sltiu(x) => {
                let val = (self.reg(x.rs) < (x.imm as u32)).into();
                self.set_reg(x.rt, val);
            }
            xori(x) => {
                let val = self.reg(x.rs) ^ (x.imm as u32);
                self.set_reg(x.rt, val);
            }
            beq(x) => {
                if self.reg(x.rs) == self.reg(x.rt) {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bgez(x) => {
                if (self.reg(x.rs) as i32) >= 0 {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bgezal(x) => {
                if (self.reg(x.rs) as i32) >= 0 {
                    self.set_reg(RegisterName::new(31), pc);
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bgtz(x) => {
                if (self.reg(x.rs) as i32) > 0 {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            blez(x) => {
                if (self.reg(x.rs) as i32) < 0 {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bltz(x) => {
                if (self.reg(x.rs) as i32) < 0 {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bltzal(x) => {
                if (self.reg(x.rs) as i32) < 0 {
                    self.set_reg(RegisterName::new(31), pc);
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            bne(x) => {
                if self.reg(x.rs) != self.reg(x.rt) {
                    pc = pc.wrapping_add(branch_offset(x));
                }
            }
            lb(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, self.arch.mem.read_u8(addr) as i8 as i32 as u32);
            }
            lbu(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, self.arch.mem.read_u8(addr) as u32);
            }
            lh(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, self.arch.mem.read_u16(addr) as i16 as i32 as u32);
            }
            lhu(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, self.arch.mem.read_u16(addr) as u32);
            }
            lw(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.set_reg(x.rt, self.arch.mem.read_u32(addr));
            }
            sb(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.arch.mem.write_u8(addr, self.reg(x.rt) as u8);
            }
            sh(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.arch.mem.write_u16(addr, self.reg(x.rt) as u16);
            }
            sw(x) => {
                let addr = self.reg(x.rs).wrapping_add(x.imm as i16 as i32 as u32);
                self.arch.mem.write_u32(addr, self.reg(x.rt));
            }
            j(x) => {
                let addr = (pc & 0xf000_0000) | ((x.target & 0x3ff_ffff) << 2);
                pc = addr;
            }
            jal(x) => {
                let addr = (pc & 0xf000_0000) | ((x.target & 0x3ff_ffff) << 2);
                self.set_reg(RegisterName::new(31), pc);
                pc = addr;
            }
            jalr(x) => {
                let addr = self.reg(x.rs);
                self.set_reg(x.rd, pc);
                pc = addr;
            }
            jr(x) => {
                pc = self.reg(x.rs);
            }
            syscall(_) => {
                self.handle_syscall();
            }
            invalid(x) => {
                return InvalidInstructionSnafu { ins: x }.fail();
            }
        }

        self.arch.set_pc(pc);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assembler::assemble;
    use crate::memory::{create_memory, EndianMode};

    const TEXT_ADDR: u32 = 0x00400024;

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
        state.arch.reg[16] = 1;
        state.arch.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 3);

        state.arch.reg[19] = i32::MAX as u32;
        state.arch.reg[20] = 1;
        state.step().unwrap_err(); //TODO: Should I check if it is InterpreterError::ArithmeticOverflow?
        assert_eq!(state.arch.reg[18], 3);
    }

    #[test]
    fn sub() {
        let mut state =
            init_state(".text\nsub $18, $16, $17\nsub $18, $16, $17\nsub $18, $16, $17");
        state.arch.reg[16] = 3;
        state.arch.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 1);

        state.arch.reg[16] = 1;
        state.arch.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], -1_i32 as u32);

        state.arch.reg[16] = i32::MIN as u32;
        state.arch.reg[17] = 1;
        state.step().unwrap_err(); //TODO: Should I check if it is InterpreterError::ArithmeticOverflow?
        assert_eq!(state.arch.reg[18], -1_i32 as u32);
    }

    #[test]
    fn and_or() {
        let mut state = init_state(".text\nand $18, $16, $17\nor $18, $16, $17");
        state.arch.reg[16] = 13;
        state.arch.reg[17] = 9;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 13 & 9);
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 13 | 9);
    }

    #[test]
    fn slt() {
        let mut state = init_state(
            ".text\nslt $18, $16, $17\nslt $18, $16, $17\nslt $18, $16, $17\nslt $18, $16, $17",
        );
        state.arch.reg[16] = 1;
        state.arch.reg[17] = 2;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 1);

        state.arch.reg[16] = 2;
        state.arch.reg[17] = 1;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 0);

        state.arch.reg[16] = 1;
        state.arch.reg[17] = -1_i32 as u32;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 0);

        state.arch.reg[16] = -2_i32 as u32;
        state.arch.reg[17] = -1_i32 as u32;
        state.step().unwrap();
        assert_eq!(state.arch.reg[18], 1);
    }

    #[test]
    fn mem() {
        let mut state = init_state(".data 0x10008000\n.word -1234, 1234\n.text\nlw $16, 0($gp)\nlw $16, 4($gp)\nadd $16, $16, $17\nsw $16, 8($gp)");
        assert_eq!(state.arch.mem.read_u32(0x10008004), 1234);
        state.arch.reg[17] = 1;

        state.step().unwrap();
        assert_eq!(state.arch.reg[16], -1234_i32 as u32);

        state.step().unwrap();
        assert_eq!(state.arch.reg[16], 1234);

        state.step().unwrap();
        assert_eq!(state.arch.reg[16], 1235);

        state.step().unwrap();
        assert_eq!(state.arch.mem.read_u32(0x10008008), 1235);
    }

    #[test]
    fn jump() {
        let mut state = init_state(".text\nj 0x00001234");
        state.step().unwrap();
        assert_eq!(state.arch.pc(), 0x1234);
    }

    #[test]
    fn beq() {
        let mut state = init_state(
            ".text\nstart:\nadd $16, $16, $17\nbeq $16, $0, fin\nj start\nfin:\nj 0x00001234",
        );
        state.arch.reg[16] = 3;
        state.arch.reg[17] = -1_i32 as u32;

        let expected_pc = [0, 4, 8, 0, 4, 8, 0, 4, 12];

        for pc in expected_pc {
            assert_eq!(pc + TEXT_ADDR, state.arch.pc());
            state.step().unwrap();
        }
        assert_eq!(state.arch.pc(), 0x1234);
    }

    #[test]
    fn arithmetic() {
        let mut state = init_state(
            r".text
            add $16, $16, $17
            sub $16, $17, $16
            addu $16, $16, $17
            and $16, $16, $17
            nor $16, $16, $17
            or $16, $16, $17
            xor $16, $16, $17
            sll $16, $16, 3
            sllv $16, $17, $18
            sra $16, $16, 3
            srav $16, $16, $18
            srl $16, $16, 3
            srlv $16, $16, $18
            slt $8, $16, $17
            slt $9, $17, $16
            sltu $10, $16, $17
            sltu $11, $17, $16",
        );

        state.arch.reg[16] = 1234;
        state.arch.reg[17] = 4321;
        state.arch.reg[18] = 2;

        let expected_output = [
            0x15b3,
            0xffff_fb2e,
            0xc0f,
            1,
            0xffff_ef1e,
            0xffff_ffff,
            0xffff_ef1e,
            0xffff_78f0,
            0x4384,
            0x870,
            0x21c,
            0x43,
            0x10,
            0x10,
            0x10,
            0x10,
            0x10,
        ];

        for output in expected_output {
            state.step().unwrap();
            assert_eq!(state.arch.reg[16], output);
        }

        assert_eq!(state.arch.reg[8], 1);
        assert_eq!(state.arch.reg[9], 0);
        assert_eq!(state.arch.reg[10], 1);
        assert_eq!(state.arch.reg[11], 0);
    }

    #[test]
    fn fibonacci() {
        // See https://gist.github.com/libertylocked/068b118354539a8be992
        let mut state = init_state(
            r"
        .text
        .globl main
        main:
            ori $a0, $0, 10
            or $s0, $ra, $zero
            jal fibonacci
            or $ra, $s0, $zero
            jr $ra  # Terminate the program
        fibonacci:
            # Prologue
            addi $sp, $sp, -12
            sw $ra, 8($sp)
            sw $s0, 4($sp)
            sw $s1, 0($sp)
            or $s0, $a0, $zero
            ori $v0, $zero, 1 # return value for terminal condition
            slti $t0, $16, 3
            bne $t0, $0, fibonacciExit # check terminal condition
            addi $a0, $s0, -1 # set args for recursive call to f(n-1)
            jal fibonacci
            or $s1, $v0, $zero # store result of f(n-1) to s1
            addi $a0, $s0, -2 # set args for recursive call to f(n-2)
            jal fibonacci
            add $v0, $s1, $v0 # add result of f(n-1) to it
        fibonacciExit:
            # Epilogue
            lw $ra, 8($sp)
            lw $s0, 4($sp)
            lw $s1, 0($sp)
            addi $sp, $sp, 12
            jr $ra
            ## End of function fibonacci",
        );

        while state.arch.pc() != 0 {
            state.step().unwrap();
        }
        assert_eq!(state.arch.reg[2], 55);
    }
}
