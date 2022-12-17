#[derive(Default, Debug)]
pub struct IfPipe {
    pub npc: u32,
    pub inst: u32,
    pub debug_pc: Option<u32>,
}

#[derive(Default, Debug)]
pub struct IdPipe {
    pub npc: u32,
    pub data_a: u32,
    pub data_b: u32,
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub rd: u32,
    pub imm: u32,
    pub ctr_unit: CtrFlags,
    pub branch_target: u32,
    pub branch_taken: bool,
    pub jump_target: u32,
    pub jump_taken: bool,
    pub debug_pc: Option<u32>,
}

#[derive(Default, Debug)]
pub struct ExPipe {
    pub alu_out: u32,
    pub data_b: u32,
    pub rd: u32,
    pub ctr_unit: CtrFlags,
    pub debug_pc: Option<u32>,
}

#[derive(Default, Debug)]
pub struct MemPipe {
    pub lmd: u32,
    pub alu_out: u32,
    pub rd: u32,
    pub ctr_unit: CtrFlags,
    pub debug_pc: Option<u32>,
}

#[derive(Default, Debug)]
pub struct WbPipe {
    pub rd: u32,
    pub data: u32,
    pub ctr_unit: CtrFlags,
    pub debug_pc: Option<u32>,
}

#[derive(Default, Debug, Clone)]
pub struct CtrFlags {
    pub reg_dst: bool,
    pub reg_write: bool,
    pub alu_src: bool,
    pub alu_op: u32,
    pub mem_to_reg: bool,
    pub mem_read: bool,
    pub mem_write: bool,
    pub branch: bool,
    pub jump: bool,
    pub if_flush: bool,
}
