pub struct IfPipe{
    pub npc: u32,
    pub inst: u32
}
impl Default for IfPipe {
    fn default() -> IfPipe {
        IfPipe {
            npc: 0x00000000,
            inst: 0x00000000
        }
    }
}

pub struct IdPipe{
    pub npc: u32,
    pub data1: u32,
    pub data2: u32,
    pub rt: u32,
    pub rd: u32,
    pub imm: u32
}
impl Default for IdPipe {
    fn default() -> IdPipe {
        IdPipe {
            npc: 0x00000000,
            data1: 0x00000000,
            data2: 0x00000000,
            rt: 0x00000000,
            rd: 0x00000000,
            imm: 0x00000000
        }
    }
}

pub struct ExPipe{
    pub branch_tgt: u32,
    pub zero: u32,
    pub aluout: u32,
    pub data2: u32,
    pub rd: u32
}
impl Default for ExPipe {
    fn default() -> ExPipe {
        ExPipe {
            branch_tgt: 0x00000000,
            zero: 0x00000000,
            aluout: 0x00000000,
            data2: 0x00000000,
            rd: 0x00000000
        }
    }
}

pub struct MemPipe{
    pub lmd: u32,
    pub aluout: u32,
    pub rd: u32
}
impl Default for MemPipe {
    fn default() -> MemPipe {
        MemPipe {
            lmd: 0x00000000,
            aluout: 0x00000000,
            rd: 0x00000000
        }
    }
}

