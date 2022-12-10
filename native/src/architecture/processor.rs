use super::pipes;
use super::stage;
use super::units;
use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

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
    if_id: pipes::IfPipe,
    id_ex: pipes::IdPipe,
    ex_mem: pipes::ExPipe,
    mem_wb: pipes::MemPipe,
    wb: pipes::WbPipe,
    is_hazard: bool,
    cur_addr: u32,
    cur_line: u32,
}
impl Processor {
    pub fn new() -> Processor {
        Processor {
            pc: 0x00400000,
            hi: 0x0,
            lo: 0x0,
            instructions: Vec::new(),
            memory: [0x0; 65536],
            registers: [0x0; 32],
            if_id: { pipes::IfPipe::default() },
            id_ex: { pipes::IdPipe::default() },
            ex_mem: { pipes::ExPipe::default() },
            mem_wb: { pipes::MemPipe::default() },
            wb: { pipes::WbPipe::default() },
            is_hazard: false,
            cur_addr: 0x00000000,
            cur_line: 0,
        }
    }

    pub fn add_instruction(&mut self, instruction: Block) {
        self.instructions.push(instruction);
    }

    pub fn next(&mut self) {
        self.cur_addr = self.pc;
        self.pc += 4;
        

        let wb_tup = stage::wb_stage::next(&mut self.mem_wb, self.registers);
        self.wb = wb_tup.0;
        self.registers = wb_tup.1;
        let mem_tup = stage::mem_stage::next(&mut self.ex_mem, self.memory);
        self.mem_wb = mem_tup.0;
        self.memory = mem_tup.1;
        self.ex_mem = stage::ex_stage::next(&mut self.id_ex);
        self.id_ex = stage::id_stage::id_next(&mut self.if_id, self.is_hazard, self.registers);
        let mut rdr_inst = Cursor::new(self.instructions[self.cur_line as usize].data.clone());
        let cur_inst = rdr_inst.read_u32::<NativeEndian>().unwrap();
        self.if_id = stage::if_stage::if_next(self.pc, cur_inst, self.is_hazard, self.cur_addr);

        if self.if_id.ran == self.cur_addr {
            self.cur_line += 1;
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
        assert_eq!(proc.if_id.inst, 0x00000000);
    }
    #[test]
    fn processor_cycle_2_inst() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x00000000).unwrap();
        proc.add_instruction(Block { data: wtr });
        proc.next();
        let mut wtr2 = vec![];
        wtr2.write_u32::<NativeEndian>(0x00400000).unwrap();
        proc.add_instruction(Block { data: wtr2 });
        assert_eq!(proc.if_id.inst, 0x00000000);
        proc.next();
        assert_eq!(proc.if_id.inst, 0x00400000);
    }
    #[test]
    fn stage_id_control_unit() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x02114020).unwrap();
        proc.add_instruction(Block { data: wtr });
        proc.next();
        let test: u32 = 0x02114020;
        let sample = units::control_unit::ctrl_unit((test & 0xFC000000) >> 26);

        assert_eq!(proc.id_ex.ctr_unit.reg_dst, sample.reg_dst);
        assert_eq!(proc.id_ex.ctr_unit.branch, sample.branch);
        assert_eq!(proc.id_ex.ctr_unit.mem_read, sample.mem_read);
        assert_eq!(proc.id_ex.ctr_unit.mem_to_reg, sample.mem_to_reg);
        assert_eq!(proc.id_ex.ctr_unit.alu_op, sample.alu_op);
        assert_eq!(proc.id_ex.ctr_unit.mem_write, sample.mem_write);
        assert_eq!(proc.id_ex.ctr_unit.alu_src, sample.alu_src);
        assert_eq!(proc.id_ex.ctr_unit.reg_write, sample.reg_write);
    }
    #[test]
    fn stage_id_datas() {
        let mut proc = Processor::new();
        let mut wtr = vec![];
        wtr.write_u32::<NativeEndian>(0x02114020).unwrap();
        proc.add_instruction(Block { data: wtr });
        proc.next();
        let test: u32 = 0x02114020;
        let sample = units::control_unit::ctrl_unit((test & 0xFC000000) >> 26);

        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        
    }
}
