use crate::component::RegisterName;
use crate::executor::pipeline::pipes;
use crate::executor::Arch;

pub fn forward_value(
    target: u32,
    arch: &Arch,
    mem_input: &pipes::ExPipe,
    wb_input: &pipes::MemPipe,
) -> u32 {
    if mem_input.ctr_unit.reg_write && mem_input.rd == target {
        mem_input.alu_out
    } else if wb_input.ctr_unit.reg_write && wb_input.rd == target {
        if wb_input.ctr_unit.mem_to_reg {
            wb_input.lmd
        } else {
            wb_input.alu_out
        }
    } else {
        arch.reg(RegisterName::new(target as u8))
    }
}
