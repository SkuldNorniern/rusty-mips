use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

use super::stage;

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
    if_id: HashMap<String, Vec<u8>>,
    id_ex: HashMap<String, Vec<u8>>,
    ex_mem: HashMap<String, Vec<u8>>,
    mem_wb: HashMap<String, Vec<u8>>,
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
            if_id: HashMap::new(),
            id_ex: HashMap::new(),
            ex_mem: HashMap::new(),
            mem_wb: HashMap::new(),
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
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(self.pc).unwrap();
        if self.if_id.is_empty()  == false {
            self.id_ex = stage::if_id_stage(&mut self.if_id);
            self.if_id.drain();
        }
        if self.is_hazard == false {
            self.if_id.insert(String::from("NPC") ,wtr);
            self.if_id.insert(String::from("INST"), self.instructions[self.cur_line as usize].data.clone());
            self.cur_line = self.cur_line + 1;
        }
        self.is_hazard = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x00000000).unwrap();
        proc.add_instruction(Block { data: wtr });
        proc.next();
        assert_eq!(proc.pc, 0x00400004);
    }
    #[test]
    fn test2() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x00000000).unwrap();

        proc.add_instruction(Block { data: wtr });
        proc.next();
        let mut rdrins = Cursor::new(proc.if_id.get("INST").unwrap());
        assert_eq!(
            rdrins.read_u32::<NativeEndian>().unwrap(),
            0x00000000
        );
    }
    #[test]
    fn test4() {
        let mut proc = Processor::new();
        proc.add_instruction(Block {
            data: vec![0x00, 0x00, 0x00, 0x00],
        });
        proc.next();
        proc.add_instruction(Block {
            data: vec![0x00, 0x00, 0x40, 0x00],
        });
        let mut rdrins = Cursor::new(proc.if_id.get("INST").unwrap());
        assert_eq!(
            rdrins.read_u32::<NativeEndian>().unwrap(),
            0x00000000
        );
        proc.next();
        ;
        let mut rdrins = Cursor::new(proc.if_id.get("INST").unwrap());
        assert_eq!(
            rdrins.read_u32::<NativeEndian>().unwrap(),
            0x00400000
        );
    }
}
