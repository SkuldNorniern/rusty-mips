#[derive(Default, Debug)]
pub struct IfPipe {
    pub npc: u32,
    pub inst: u32,
    pub ran: u32,
}

#[derive(Default, Debug)]
pub struct CtrUnitFull {
    pub reg_dst: u32,
    pub reg_write: u32,
    pub alu_src: u32,
    pub alu_op: u32,
    pub mem_to_reg: u32,
    pub mem_read: u32,
    pub mem_write: u32,
    pub branch: u32,
    pub if_flush: u32,
}

#[derive(Default, Debug)]
pub struct IdPipe {
    pub npc: u32,
    pub data_a: u32,
    pub data_b: u32,
    pub rs: u32,
    pub rt: u32,
    pub rd: u32,
    pub imm: u32,
    pub ctr_unit: CtrUnitFull,
    pub ran: u32,
}

#[derive(Default, Debug)]
pub struct CtrUnitSlim {
    pub reg_write: u32,
    pub mem_to_reg: u32,
    pub mem_read: u32,
    pub mem_write: u32,
    pub branch: u32,
}

#[derive(Default, Debug)]
pub struct ExPipe {
    pub branch_tgt: u32,
    pub zero: u32,
    pub alu_out: u32,
    pub data_b: u32,
    pub rd: u32,
    pub ctr_unit: CtrUnitSlim,
    pub ran: u32,
}

#[derive(Default, Debug)]
pub struct MemPipe {
    pub lmd: u32,
    pub alu_out: u32,
    pub rd: u32,
    pub ctr_unit: CtrUnitSlim,
    pub ran: u32,
}

#[derive(Default, Debug)]
pub struct WbPipe {
    pub ran: u32,
}
