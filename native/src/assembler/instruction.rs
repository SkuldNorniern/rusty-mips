use crate::component::RegisterName;

#[derive(Copy, Clone)]
pub struct FormatR {
    rd: RegisterName,
    rs: RegisterName,
    rt: RegisterName,
    shamt: u8,
}

impl FormatR {
    pub fn new(rd: RegisterName, rs: RegisterName, rt: RegisterName, shamt: u8) -> Self {
        assert_eq!(shamt & 0b1110_0000, 0, "shamt must be 5 bits");

        FormatR { rd, rs, rt, shamt }
    }

    pub fn encode(&self, funct: u8) -> u32 {
        (self.rs.num() as u32) << 21
            | (self.rt.num() as u32) << 16
            | (self.rd.num() as u32) << 11
            | (self.shamt as u32) << 6
            | (funct as u32)
    }
}

#[derive(Copy, Clone)]
pub struct FormatI {
    rs: RegisterName,
    rt: RegisterName,
    imm: u16,
}

impl FormatI {
    pub fn new(rs: RegisterName, rt: RegisterName, imm: u16) -> Self {
        FormatI { rs, rt, imm }
    }

    pub fn encode(&self, opcode: u8) -> u32 {
        (opcode as u32) << 26
            | (self.rs.num() as u32) << 21
            | (self.rt.num() as u32) << 16
            | (self.imm as u32)
    }
}

#[derive(Copy, Clone)]
pub struct FormatJ {
    target: u32,
}

impl FormatJ {
    pub(crate) fn new(target: u32) -> Self {
        assert_eq!(target & 0xfc00_0000, 0, "target must have top 6 bits zero");

        FormatJ { target }
    }

    fn encode(&self, opcode: u8) -> u32 {
        (opcode as u32) << 26 | self.target
    }
}

enum InstructionFormat<'a> {
    R(&'a FormatR),
    I(&'a FormatI),
    J(&'a FormatJ),
}

#[derive(Copy, Clone)]
pub enum Instruction {
    Unknown(u32),
    And(FormatR),
    Or(FormatR),
    Add(FormatR),
    Sub(FormatR),
    Slt(FormatR),
    Lw(FormatI),
    Sw(FormatI),
    Beq(FormatI),
    J(FormatJ),
}

impl Instruction {
    pub fn encode(&self) -> u32 {
        use InstructionFormat::*;

        // Calling x.encode directly will inline each invocation resulting in code bloat
        // opcode means funct on R-type instructions
        let (fmt, opcode) = match self {
            Instruction::Unknown(x) => return *x,
            Instruction::And(x) => (R(x), 0x24),
            Instruction::Or(x) => (R(x), 0x25),
            Instruction::Add(x) => (R(x), 0x20),
            Instruction::Sub(x) => (R(x), 0x22),
            Instruction::Slt(x) => (R(x), 0x2a),
            Instruction::Lw(x) => (I(x), 0x23),
            Instruction::Sw(x) => (I(x), 0x2b),
            Instruction::Beq(x) => (I(x), 0x4),
            Instruction::J(x) => (J(x), 0x2),
        };

        debug_assert!(opcode & 0b1100_0000 == 0, "opcode must be 6 bits");

        match fmt {
            R(x) => x.encode(opcode),
            I(x) => x.encode(opcode),
            J(x) => x.encode(opcode),
        }
    }
}
