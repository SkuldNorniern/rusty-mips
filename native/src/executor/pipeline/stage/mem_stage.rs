use crate::executor::pipeline::pipes;
use crate::executor::Arch;

pub fn mem_next(
    mem_input: &pipes::ExPipe,
    wb_input: &pipes::MemPipe,
    arch: &mut Arch,
) -> pipes::MemPipe {
    let lmd = if mem_input.ctr_unit.mem_read {
        arch.mem.read_u32(mem_input.alu_out)
    } else {
        0
    };

    if mem_input.ctr_unit.mem_write {
        let data = if wb_input.ctr_unit.mem_read && wb_input.rd == mem_input.rd {
            wb_input.lmd
        } else {
            mem_input.data_b
        };
        arch.mem.write_u32(mem_input.alu_out, data);
    }

    pipes::MemPipe {
        lmd,
        alu_out: mem_input.alu_out,
        rd: mem_input.rd,
        ctr_unit: mem_input.ctr_unit.clone(),
        debug_pc: mem_input.debug_pc,
    }
}
