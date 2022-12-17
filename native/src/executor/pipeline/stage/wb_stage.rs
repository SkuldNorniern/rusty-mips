use crate::component::RegisterName;
use crate::executor::pipeline::pipes;
use crate::executor::Arch;

pub fn wb_next(wb_input: &pipes::MemPipe, arch: &mut Arch) -> pipes::WbPipe {
    let value = if wb_input.ctr_unit.mem_to_reg {
        wb_input.lmd
    } else {
        wb_input.alu_out
    };

    if wb_input.ctr_unit.reg_write {
        arch.set_reg(RegisterName::new(wb_input.rd as u8), value);
    }

    pipes::WbPipe {
        rd: wb_input.rd,
        data: value,
        ctr_unit: wb_input.ctr_unit.clone(),
        debug_pc: wb_input.debug_pc,
    }
}
