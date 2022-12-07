pub struct IfPipe {
    pub npc: u32,
    pub inst: u32,
    pub ran: u32,
}
impl Default for IfPipe {
    fn default() -> IfPipe {
        IfPipe {
            npc: 0x00000000,
            inst: 0x00000000,
            ran: 0x00000000,
        }
    }
}

pub struct CtrUnit {
    pub reg_dst: u32,
    pub reg_write: u32,
    pub alu_src: u32,
    pub alu_op: u32,
    pub mem_to_reg: u32,
    pub mem_read: u32,
    pub mem_write: u32,
    pub branch: u32,
}
impl Default for CtrUnit {
    fn default() -> CtrUnit {
        CtrUnit {
            reg_dst: 0b0,
            reg_write: 0b0,
            alu_src: 0b0,
            alu_op: 0b0,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b0,
        }
    }
}

pub struct IdPipe {
    pub npc: u32,
    pub data_a: u32,
    pub data_b: u32,
    pub rs: u32,
    pub rt: u32,
    pub rd: u32,
    pub imm: u32,
    pub ctr_unit: CtrUnit,
    pub ran: u32,
}
impl Default for IdPipe {
    fn default() -> IdPipe {
        IdPipe {
            npc: 0x00000000,
            data_a: 0x00000000,
            data_b: 0x00000000,
            rs: 0x00000000,
            rt: 0x00000000,
            rd: 0x00000000,
            imm: 0x00000000,
            ctr_unit: CtrUnit::default(),
            ran: 0x00000000,
        }
    }
}

pub struct ExPipe {
    pub branch_tgt: u32,
    pub zero: u32,
    pub aluout: u32,
    pub data2: u32,
    pub rd: u32,
    pub ran: u32,
}
impl Default for ExPipe {
    fn default() -> ExPipe {
        ExPipe {
            branch_tgt: 0x00000000,
            zero: 0x00000000,
            aluout: 0x00000000,
            data2: 0x00000000,
            rd: 0x00000000,
            ran: 0x00000000,
        }
    }
}

pub struct MemPipe {
    pub lmd: u32,
    pub aluout: u32,
    pub rd: u32,
    pub ran: u32,
}
impl Default for MemPipe {
    fn default() -> MemPipe {
        MemPipe {
            lmd: 0x00000000,
            aluout: 0x00000000,
            rd: 0x00000000,
            ran: 0x00000000,
        }
    }
}
