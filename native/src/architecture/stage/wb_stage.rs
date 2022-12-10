use crate::architecture::pipes;

pub fn next(_mem_wb: &mut pipes::MemPipe,_reg_list: [u32;32]) -> (pipes::WbPipe,[u32;32]) {
    let mut wb = pipes::WbPipe::default();
    let mut reg_list = _reg_list;
    wb.ran = _mem_wb.ran;
    
    if _mem_wb.ctr_unit.reg_write == 1 && _mem_wb.rd != 0 {
        if _mem_wb.ctr_unit.mem_to_reg == 1 {
            reg_list[_mem_wb.rd as usize] = _mem_wb.lmd;
        } else {
            reg_list[_mem_wb.rd as usize] = _mem_wb.alu_out;
        }
    }
    (wb,reg_list)
}
