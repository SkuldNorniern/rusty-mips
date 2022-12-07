pub mod pipes;

use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use crate::stage;

pub struct Block {
    //labels: Map<String, u32>,
    data: Vec<u8>,
}
pub struct Processor {
    pc: u32,
    hi: u32,
    lo: u32,
    instructions: Vec<Block>,
    memory: [u32; 65536],
    registers: [u32; 32],
    if_id: pipes::if_pipe,
    id_ex: pipes::id_pipe,
    ex_mem: pipes::ex_pipe,
    mem_wb: pipes::mem_pipe,
    is_hazard: bool,
    cur_line: i32,
}
impl Processor {
    pub fn new() -> Processor {
        let mut proc = Processor {
            pc: 0x00400000,
            hi: 0x0,
            lo: 0x0,
            instructions: Vec::new(),
            memory: [0; 65536],
            registers: [0; 32],
            if_id: {pipes::if_pipe::default()},
            id_ex: {pipes::id_pipe::default()},
            ex_mem:{pipes::ex_pipe::default()},
            mem_wb:{pipes::mem_pipe::default()},
            is_hazard: false,
            cur_line: 0,
        };
        proc
    }

    pub fn add_instruction(&mut self, instruction: Block) {
        self.instructions.push(instruction);
    }

    pub fn next(&mut self) {
        self.pc = self.pc + 4;
        //if self.if_id.is_empty()  == false {
        //    self.id_ex = stage::if_id_stage(&mut self.if_id);
            
        //}
        if self.is_hazard == false {
            self.if_id.npc = self.pc;
            let mut rdrins = Cursor::new(self.instructions[self.cur_line as usize].data.clone());
            self.if_id.inst = rdrins.read_u32::<NativeEndian>().unwrap();
            self.cur_line = self.cur_line + 1;
        }
        self.is_hazard = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn processor_pc_value() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x00000000).unwrap();
        proc.add_instruction(Block { data: wtr });
        proc.next();
        assert_eq!(proc.pc, 0x00400004);
    }
    #[test]
    fn processor_cycle_1_inst() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x00000000).unwrap();

        proc.add_instruction(Block { data: wtr });
        proc.next();  
        assert_eq!(
            proc.if_id.inst,
            0x00000000
        );
    }
    #[test]
    fn processor_cycle_2_inst() {
        let mut proc = Processor::new();
        proc.add_instruction(Block {
            data: vec![0x00, 0x00, 0x00, 0x00],
        });
        proc.next();
        proc.add_instruction(Block {
            data: vec![0x00, 0x00, 0x40, 0x00],
        });
        assert_eq!(
            proc.if_id.inst,
            0x00000000
        );
        proc.next();
        assert_eq!(
            proc.if_id.inst,
            0x00400000
        );
    }
}
