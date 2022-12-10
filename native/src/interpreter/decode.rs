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
    and(TypeR),
    or(TypeR),
    sub(TypeR),
    slt(TypeR),

    // I format
    lw(TypeI),
    sw(TypeI),
    beq(TypeI),

    // J format
    j(TypeJ),
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
        0x24 => and(x),
        0x25 => or(x),
        0x22 => sub(x),
        0x2a => slt(x),
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
        0x23 => lw(x),
        0x2b => sw(x),
        0x4 => beq(x),
        _ => {
            return Err(UnknownOpcodeSnafu {
                opcode: opcode as u8,
            }
            .build())
        }
    })
}

fn decode_j(instruction: u32) -> Result<Instruction, InterpreterError> {
    use Instruction::*;

    debug_assert!(
        (instruction & 0xfc00_0000) >> 26 == 0x2,
        "only instruction `J` is accepted"
    );

    Ok(j(TypeJ {
        addr: instruction & 0x3ff_ffff,
    }))
}

pub fn decode(instruction: u32) -> Result<Instruction, InterpreterError> {
    let opcode = (instruction & 0xfc00_0000) >> 26;

    if opcode == 0 {
        decode_type_r(instruction)
    } else if opcode == 0x2 {
        decode_j(instruction)
    } else if opcode == 0x3 {
        todo!()
        //decode_jal(instruction)
    } else {
        decode_i(instruction, opcode)
    }
}
