use crate::component::RegisterName;
use crate::interpreter::error::{InterpreterError, UnknownFunctSnafu, UnknownOpcodeSnafu};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TypeR {
    pub rs: RegisterName,
    pub rt: RegisterName,
    pub rd: RegisterName,
    pub shamt: u8,
}

impl Default for TypeR {
    fn default() -> Self {
        TypeR {
            rs: RegisterName::new(0),
            rt: RegisterName::new(0),
            rd: RegisterName::new(0),
            shamt: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TypeI {
    pub rs: RegisterName,
    pub rt: RegisterName,
    pub imm: u16,
}

impl Default for TypeI {
    fn default() -> Self {
        TypeI {
            rs: RegisterName::new(0),
            rt: RegisterName::new(0),
            imm: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct TypeJ {
    pub addr: u32,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Instruction {
    // R format
    add(TypeR),
    addu(TypeR),
    and(TypeR),
    jr(TypeR),
    nor(TypeR),
    or(TypeR),
    slt(TypeR),
    sltu(TypeR),
    sll(TypeR),
    srl(TypeR),
    sub(TypeR),
    subu(TypeR),

    // I format
    addi(TypeI),
    addiu(TypeI),
    andi(TypeI),
    beq(TypeI),
    bne(TypeI),
    lbu(TypeI),
    lhu(TypeI),
    ll(TypeI),
    lui(TypeI),
    lw(TypeI),
    ori(TypeI),
    slti(TypeI),
    sltiu(TypeI),
    sb(TypeI),
    sc(TypeI),
    sh(TypeI),
    sw(TypeI),

    // J format
    j(TypeJ),
    jal(TypeJ),
}

fn decode_type_r(instruction: u32) -> Result<Instruction, InterpreterError> {
    debug_assert!(
        instruction & 0xfc00_0000 == 0,
        "R-type instruction must have opcode=0"
    );

    use Instruction::*;

    let funct = (instruction & 0x3f) as u8;
    let shamt = (instruction & 0x7c0) >> 6;
    let rd = (instruction & 0xf800) >> 11;
    let rt = (instruction & 0x1f_0000) >> 16;
    let rs = (instruction & 0x3e0_0000) >> 21;

    let x = TypeR {
        rs: RegisterName::new(rs as u8),
        rt: RegisterName::new(rt as u8),
        rd: RegisterName::new(rd as u8),
        shamt: shamt as u8,
    };

    Ok(match funct {
        0x20 => add(x),
        0x21 => addu(x),
        0x24 => and(x),
        0x08 => jr(x),
        0x27 => nor(x),
        0x25 => or(x),
        0x2a => slt(x),
        0x2b => sltu(x),
        0x00 => sll(x),
        0x02 => srl(x),
        0x22 => sub(x),
        0x23 => subu(x),
        _ => UnknownFunctSnafu { funct }.fail()?,
    })
}

fn decode_i(instruction: u32, opcode: u32) -> Result<Instruction, InterpreterError> {
    use Instruction::*;

    let imm = instruction & 0xffff;
    let rt = (instruction & 0x1f_0000) >> 16;
    let rs = (instruction & 0x3e0_0000) >> 21;

    let x = TypeI {
        rs: RegisterName::new(rs as u8),
        rt: RegisterName::new(rt as u8),
        imm: imm as u16,
    };

    Ok(match opcode {
        0x08 => addi(x),
        0x09 => addiu(x),
        0x0c => andi(x),
        0x04 => beq(x),
        0x05 => bne(x),
        0x24 => lbu(x),
        0x25 => lhu(x),
        0x30 => ll(x),
        0x0f => lui(x),
        0x23 => lw(x),
        0x0d => ori(x),
        0x0a => slti(x),
        0x0b => sltiu(x),
        0x28 => sb(x),
        0x38 => sc(x),
        0x29 => sh(x),
        0x2b => sw(x),
        _ => {
            return Err(UnknownOpcodeSnafu {
                opcode: opcode as u8,
            }
            .build())
        }
    })
}

fn decode_j(instruction: u32, opcode: u32) -> Result<Instruction, InterpreterError> {
    use Instruction::*;

    let x = TypeJ {
        addr: instruction & 0x3ff_ffff,
    };

    Ok(match opcode {
        0x2 => j(x),
        0x3 => jal(x),
        _ => {
            return Err(UnknownOpcodeSnafu {
                opcode: opcode as u8,
            }
            .build())
        }
    })
}

pub fn decode(instruction: u32) -> Result<Instruction, InterpreterError> {
    let opcode = (instruction & 0xfc00_0000) >> 26;

    if opcode == 0 {
        decode_type_r(instruction)
    } else if opcode == 2 || opcode == 3 {
        decode_j(instruction, opcode)
    } else {
        decode_i(instruction, opcode)
    }
}
