use crate::executor::pipeline::pipes;

pub fn next(_ex_mem: &mut pipes::ExPipe, _lmd: u32) -> (pipes::MemPipe, (u32, u32, bool)) {
    let mut mem_wb = pipes::MemPipe::default();
    let mut mem_data = (0, 0, false);
    mem_wb.ran = _ex_mem.ran;

    mem_wb.ctr_unit.mem_to_reg = _ex_mem.ctr_unit.mem_to_reg;
    mem_wb.ctr_unit.reg_write = _ex_mem.ctr_unit.reg_write;

    if _ex_mem.ctr_unit.mem_read == 1 {
        mem_wb.lmd = _lmd;
    }

    if _ex_mem.ctr_unit.mem_write == 1 {
        mem_data = ((_ex_mem.alu_out), _ex_mem.data_b, true);
    }

    mem_wb.alu_out = _ex_mem.alu_out;
    mem_wb.rd = _ex_mem.rd;

    (mem_wb, mem_data)
}
