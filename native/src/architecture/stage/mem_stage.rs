use crate::architecture::pipes;

pub fn next(
    _ex_mem: &mut pipes::ExPipe,
    _data_list: [u32; 65536],
) -> (pipes::MemPipe, [u32; 65536]) {
    let mut mem_wb = pipes::MemPipe::default();
    let mut data_list = _data_list;
    mem_wb.ran = _ex_mem.ran;

    mem_wb.ctr_unit.mem_read = _ex_mem.ctr_unit.mem_read;
    mem_wb.ctr_unit.reg_write = _ex_mem.ctr_unit.reg_write;

    if _ex_mem.ctr_unit.mem_read == 1 {
        mem_wb.lmd = _data_list[(_ex_mem.alu_out / 4) as usize];
    }

    if _ex_mem.ctr_unit.mem_write == 1 {
        data_list[(_ex_mem.alu_out / 4) as usize] = _ex_mem.data_b;
    }

    mem_wb.alu_out = _ex_mem.alu_out;
    mem_wb.rd = _ex_mem.rd;

    (mem_wb, data_list)
}
