use super::pipes;
use super::stage;
use super::units;
use crate::component::RegisterName;

use crate::executor::Arch;
use crate::memory::Memory;

emum FinOption{
    Final(bool)
}
use FinOption::Final;

#[derive(Debug)]
pub struct Pipeline {
    arch: Arch,
    if_id: pipes::IfPipe,
    id_ex: pipes::IdPipe,
    ex_mem: pipes::ExPipe,
    mem_wb: pipes::MemPipe,
    wb: pipes::WbPipe,
    fwd_unit: units::forward_unit::FwdUnit,
}

impl Pipeline {
    pub fn new(mem: Box<dyn Memory>) -> Pipeline {
        Pipeline {
            arch: Arch::new(mem),
            if_id: { pipes::IfPipe::default() },
            id_ex: { pipes::IdPipe::default() },
            ex_mem: { pipes::ExPipe::default() },
            mem_wb: { pipes::MemPipe::default() },
            wb: { pipes::WbPipe::default() },
            fwd_unit: { units::forward_unit::FwdUnit::default() },
        }
    }

    pub fn as_arch(&self) -> &Arch {
        &self.arch
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        &mut self.arch
    }

    pub fn into_arch(mut self) -> Arch {
        self.finalize();
        self.arch
    }

    pub fn reg(&self, name: u32) -> u32 {
        assert!(name < 32, "register name must be under 32");
        self.arch.reg(RegisterName::new(name as u8))
    }

    pub fn set_reg(&mut self, name: u32, val: u32) {
        assert!(name < 32, "register name must be under 32");
        self.arch.set_reg(RegisterName::new(name as u8), val);
    }

    pub fn step(&mut self, _finalize: &[FinOption]) {
        let mut finalize = false;
        match _finalize[0] {
            Final(f) => finalize = *f,
        }
        self.fwd_unit = stage::forward::fwd_ctrl(
            &mut self.ex_mem,
            &mut self.mem_wb,
            &mut self.id_ex,
            self.fwd_unit,
        );
        let wb_tup = stage::wb_stage::next(&mut self.mem_wb);
        self.wb = wb_tup.0;
        self.set_reg(wb_tup.1 .0, wb_tup.1 .1);
        let lmd_addr = self.ex_mem.alu_out / 4;
        let mem_tup = stage::mem_stage::next(&mut self.ex_mem, self.arch.mem.read_u32(lmd_addr));
        self.mem_wb = mem_tup.0;
        if mem_tup.1 .2 {
            self.arch.mem.write_u32(mem_tup.1 .0, mem_tup.1 .1);
        }
        self.ex_mem = stage::ex_stage::next(&mut self.id_ex, self.fwd_unit);
        self.id_ex =
            stage::id_stage::id_next(&mut self.if_id, self.fwd_unit.hazard, self.arch.regs());
        let mut cur_inst = self.arch.mem.read_u32(self.arch.pc());
        if _finalize:
            curinst = 0x00000000;
        let if_tup =
            stage::if_stage::if_next(cur_inst, self.fwd_unit, &mut self.ex_mem, self.arch.pc(),finalize);
        self.if_id = if_tup.0;
        self.arch.set_pc(if_tup.1);
        self.fwd_unit = stage::hazard::hazard_ctrl(&mut self.if_id, &mut self.id_ex, self.fwd_unit);
    }

    pub fn finalize(&mut self) {
        //TODO: execute everything in current pipeline, filling with bubble
        self.step(&[Final(true)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    use crate::memory::{create_memory, EndianMode};

    fn make(asm: &str) -> Pipeline {
        let native = EndianMode::native();
        let segs = assemble(native, asm).unwrap();
        let mem = create_memory(native, &segs);
        Pipeline::new(mem)
    }

    #[test]
    fn processor_pc_value() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.arch.pc(), 0x00400028);
    }

    #[test]
    fn processor_cycle_1_inst() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
    }

    #[test]
    fn processor_cycle_2_inst() {
        let mut proc = make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();

        assert_eq!(proc.if_id.inst, 0x02001022);
    }

    #[test]
    fn processor_cycle_3_inst() {
        let mut proc = make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02001022);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x0);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        assert_eq!(proc.ex_mem.alu_out, 0x9020);
    }

    #[test]
    fn processor_cycle_4_inst() {
        //TODO Finish this test
        let mut proc = make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero\nadd $17, $4, $5");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02001022);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x00858820);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        assert_eq!(proc.ex_mem.alu_out, 0x9020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x0);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        //assert_eq!(proc.ex_mem.alu_out, );
        //assert_eq!(proc.mem_wb.alu_out, );
    }

    #[test]
    fn processor_cycle_5_inst() {
        //TODO Finish this test
        let mut proc =
            make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero\nadd $17, $4, $5\nadd $6, $7, $8");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02001022);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x00858820);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        assert_eq!(proc.ex_mem.alu_out, 0x9020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x00E83020);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        //assert_eq!(proc.ex_mem.alu_out, );
        //assert_eq!(proc.mem_wb.alu_out, );
        proc.step();
        assert_eq!(proc.if_id.inst, 0x0);
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
        //assert_eq!(proc.ex_mem.alu_out, );
        //assert_eq!(proc.mem_wb.alu_out, );
        assert_eq!(proc.reg(18), 0x0);
    }

    //TODO Add more tests
    #[test]
    fn stage_id_control_unit() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
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
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
    }
}
