use super::register_name::RegisterName;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TypeR {
    pub rs: RegisterName,
    pub rt: RegisterName,
    pub rd: RegisterName,
    pub shamt: u8,
}

impl TypeR {
    pub fn encode(&self, funct: u8) -> u32 {
        (self.rs.num() as u32) << 21
            | (self.rt.num() as u32) << 16
            | (self.rd.num() as u32) << 11
            | (self.shamt as u32) << 6
            | (funct as u32)
    }

    fn decode_unchecked(ins: u32) -> (u8, TypeR) {
        let rs = ((ins >> 21) & 0x1f) as u8;
        let rt = ((ins >> 16) & 0x1f) as u8;
        let rd = ((ins >> 11) & 0x1f) as u8;
        let shamt = ((ins >> 6) & 0x1f) as u8;
        let funct = (ins & 0x3f) as u8;

        (
            funct,
            TypeR {
                rs: RegisterName::new(rs),
                rt: RegisterName::new(rt),
                rd: RegisterName::new(rd),
                shamt,
            },
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TypeI {
    pub rs: RegisterName,
    pub rt: RegisterName,
    pub imm: u16,
}

impl TypeI {
    pub fn encode(&self, opcode: u8) -> u32 {
        (opcode as u32) << 26
            | (self.rs.num() as u32) << 21
            | (self.rt.num() as u32) << 16
            | (self.imm as u32)
    }

    fn decode_unchecked(ins: u32) -> (u8, TypeI) {
        let opcode = ((ins >> 26) & 0x3f) as u8;
        let rs = ((ins >> 21) & 0x1f) as u8;
        let rt = ((ins >> 16) & 0x1f) as u8;
        let imm = (ins & 0xffff) as u16;

        (
            opcode,
            TypeI {
                rs: RegisterName::new(rs),
                rt: RegisterName::new(rt),
                imm,
            },
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TypeJ {
    pub target: u32,
}

impl TypeJ {
    pub fn encode(&self, opcode: u8) -> u32 {
        (opcode as u32) << 26 | self.target
    }

    fn decode_unchecked(ins: u32) -> (u8, TypeJ) {
        let opcode = ((ins >> 26) & 0x3f) as u8;
        let target = ins & 0x03ff_ffff;

        (opcode, TypeJ { target })
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Instruction {
    // Most MIPS I opcodes: https://opencores.org/projects/plasma/opcodes
    // Arithmetic - Core
    add(TypeR),
    addu(TypeR),
    and(TypeR),
    nor(TypeR),
    or(TypeR),
    slt(TypeR),
    sltu(TypeR),
    sub(TypeR),
    subu(TypeR),
    xor(TypeR),

    // Arithmetic - Shifts
    sll(TypeR),
    sllv(TypeR),
    sra(TypeR),
    srav(TypeR),
    srl(TypeR),
    srlv(TypeR),

    // Arithmetic - immediate
    addi(TypeI),
    addiu(TypeI),
    andi(TypeI),
    lui(TypeI),
    ori(TypeI),
    slti(TypeI),
    sltiu(TypeI),
    xori(TypeI),

    // Branch
    beq(TypeI),
    bgez(TypeI),
    bgezal(TypeI),
    bgtz(TypeI),
    blez(TypeI),
    bltz(TypeI),
    bltzal(TypeI),
    bne(TypeI),

    // Memory access
    lb(TypeI),
    lbu(TypeI),
    lh(TypeI),
    lhu(TypeI),
    lw(TypeI),
    sb(TypeI),
    sh(TypeI),
    sw(TypeI),

    // Jump & Extra
    j(TypeJ),
    jal(TypeJ),
    jalr(TypeR),
    jr(TypeR),
    syscall(TypeR),
    invalid(u32),
}

impl Instruction {
    pub fn as_invalid(&self) -> Option<u32> {
        if let Instruction::invalid(x) = self {
            Some(*x)
        } else {
            None
        }
    }
}

enum TypeGroup {
    R(TypeR),
    I(TypeI),
    J(TypeJ),
}

impl Instruction {
    pub fn encode(&self) -> u32 {
        use Instruction::*;
        use TypeGroup::*;

        // this variable `opcode` means `funct` on R-type instructions
        let (opcode, ty) = match self {
            add(x) => (0x20, R(*x)),
            addu(x) => (0x21, R(*x)),
            and(x) => (0x24, R(*x)),
            nor(x) => (0x27, R(*x)),
            or(x) => (0x25, R(*x)),
            slt(x) => (0x2a, R(*x)),
            sltu(x) => (0x2b, R(*x)),
            sub(x) => (0x22, R(*x)),
            subu(x) => (0x23, R(*x)),
            xor(x) => (0x26, R(*x)),
            sll(x) => (0x00, R(*x)),
            sllv(x) => (0x04, R(*x)),
            sra(x) => (0x03, R(*x)),
            srav(x) => (0x07, R(*x)),
            srl(x) => (0x02, R(*x)),
            srlv(x) => (0x06, R(*x)),

            addi(x) => (0x08, I(*x)),
            addiu(x) => (0x09, I(*x)),
            andi(x) => (0x0c, I(*x)),
            lui(x) => (0x0f, I(*x)),
            ori(x) => (0x0d, I(*x)),
            slti(x) => (0x0a, I(*x)),
            sltiu(x) => (0x0b, I(*x)),
            xori(x) => (0x0e, I(*x)),

            beq(x) => (0x04, I(*x)),
            bgez(x) => (
                0x01,
                I(TypeI {
                    rt: RegisterName::new(0b00001),
                    ..*x
                }),
            ),
            bgezal(x) => (
                0x01,
                I(TypeI {
                    rt: RegisterName::new(0b10001),
                    ..*x
                }),
            ),
            bgtz(x) => (
                0x07,
                I(TypeI {
                    rt: RegisterName::new(0),
                    ..*x
                }),
            ),
            blez(x) => (
                0x06,
                I(TypeI {
                    rt: RegisterName::new(0),
                    ..*x
                }),
            ),
            bltz(x) => (
                0x01,
                I(TypeI {
                    rt: RegisterName::new(0b00000),
                    ..*x
                }),
            ),
            bltzal(x) => (
                0x01,
                I(TypeI {
                    rt: RegisterName::new(0b10000),
                    ..*x
                }),
            ),
            bne(x) => (0x05, I(*x)),

            lb(x) => (0x20, I(*x)),
            lbu(x) => (0x24, I(*x)),
            lh(x) => (0x21, I(*x)),
            lhu(x) => (0x25, I(*x)),
            lw(x) => (0x23, I(*x)),
            sb(x) => (0x28, I(*x)),
            sh(x) => (0x29, I(*x)),
            sw(x) => (0x2b, I(*x)),

            j(x) => (2, J(*x)),
            jal(x) => (3, J(*x)),
            jalr(x) => (0x09, R(*x)),
            jr(x) => (0x08, R(*x)),
            syscall(x) => (0x0c, R(*x)),
            invalid(x) => return *x,
        };

        match ty {
            R(x) => x.encode(opcode),
            I(x) => x.encode(opcode),
            J(x) => x.encode(opcode),
        }
    }

    pub fn decode(ins: u32) -> Self {
        use Instruction::*;

        let opcode = ((ins >> 26) & 0x3f) as u8;

        let r = TypeR::decode_unchecked(ins).1;
        let i = TypeI::decode_unchecked(ins).1;
        let tj = TypeJ::decode_unchecked(ins).1;

        match opcode {
            0x00 => {
                let funct = (ins & 0x3f) as u8;
                match funct {
                    0x20 => add(r),
                    0x21 => addu(r),
                    0x24 => and(r),
                    0x27 => nor(r),
                    0x25 => or(r),
                    0x2a => slt(r),
                    0x2b => sltu(r),
                    0x22 => sub(r),
                    0x23 => subu(r),
                    0x26 => xor(r),
                    0x00 => sll(r),
                    0x04 => sllv(r),
                    0x03 => sra(r),
                    0x07 => srav(r),
                    0x02 => srl(r),
                    0x06 => srlv(r),
                    0x09 => jalr(r),
                    0x08 => jr(r),
                    0x0c => syscall(r),
                    _ => invalid(ins),
                }
            }
            0x08 => addi(i),
            0x09 => addiu(i),
            0x0c => andi(i),
            0x0f => lui(i),
            0x0d => ori(i),
            0x0a => slti(i),
            0x0b => sltiu(i),
            0x0e => xori(i),
            0x04 => beq(i),
            0x01 => match i.rt.num() {
                0b00000 => bltz(i),
                0b00001 => bgez(i),
                0b10000 => bltzal(i),
                0b10001 => bgezal(i),
                _ => invalid(ins),
            },
            0x07 => {
                if i.rt.num() == 0 {
                    bgtz(i)
                } else {
                    invalid(ins)
                }
            }
            0x06 => {
                if i.rt.num() == 0 {
                    blez(i)
                } else {
                    invalid(ins)
                }
            }
            0x05 => bne(i),
            0x20 => lb(i),
            0x24 => lbu(i),
            0x21 => lh(i),
            0x25 => lhu(i),
            0x23 => lw(i),
            0x28 => sb(i),
            0x29 => sh(i),
            0x2b => sw(i),
            2 => j(tj),
            3 => jal(tj),
            _ => invalid(ins),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rayon::prelude::*;

    #[test]
    #[ignore] // very expensive and parallelized test. would take an hour in a weak machine
    fn decode_then_encode() {
        let processed = (0..=0xffff_ffff)
            .into_par_iter()
            .map(|x| {
                assert_eq!(x, Instruction::decode(x).encode());
            })
            .count();

        assert_eq!(processed, 4294967296);
    }
}
