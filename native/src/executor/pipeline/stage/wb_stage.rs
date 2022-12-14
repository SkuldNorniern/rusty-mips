use crate::executor::pipeline::pipes;

pub fn next(_mem_wb: &mut pipes::MemPipe) -> (pipes::WbPipe, (u32, u32)) {
    let mut wb = pipes::WbPipe::default();
    let mut reg_data = (0, 0);
    wb.ran = _mem_wb.ran;

    if _mem_wb.ctr_unit.reg_write == 1 && _mem_wb.rd != 0 {
        if _mem_wb.ctr_unit.mem_to_reg == 1 {
            reg_data = (_mem_wb.rd, _mem_wb.lmd);
        } else {
            reg_data = (_mem_wb.rd, _mem_wb.alu_out);
        }
    }
    (wb, reg_data)
}
