pub struct if_pipe{
    pub npc: u32,
    pub inst: u32
}
impl Default for if_pipe {
    fn default() -> if_pipe {
        if_pipe {
            npc: 0x00000000,
            inst: 0x00000000
        }
    }
}

pub struct id_pipe{
    pub npc: u32,
    pub data1: u32,
    pub data2: u32,
    pub rt: u32,
    pub rd: u32,
    pub imm: u32
}
impl Default for id_pipe {
    fn default() -> id_pipe {
        id_pipe {
            npc: 0x00000000,
            data1: 0x00000000,
            data2: 0x00000000,
            rt: 0x00000000,
            rd: 0x00000000,
            imm: 0x00000000
        }
    }
}

pub struct ex_pipe{
    pub branch_tgt: u32,
    pub zero: u32,
    pub aluout: u32,
    pub data2: u32,
    pub rd: u32
}
impl Default for ex_pipe {
    fn default() -> ex_pipe {
        ex_pipe {
            branch_tgt: 0x00000000,
            zero: 0x00000000,
            aluout: 0x00000000,
            data2: 0x00000000,
            rd: 0x00000000
        }
    }
}

pub struct mem_pipe{
    pub lmd: u32,
    pub aluout: u32,
    pub rd: u32
}
impl Default for mem_pipe {
    fn default() -> mem_pipe {
        mem_pipe {
            lmd: 0x00000000,
            aluout: 0x00000000,
            rd: 0x00000000
        }
    }
}

