use crate::disassembler::disassemble;
use crate::executor::Arch;
use crate::executor::pipeline::{info, pipes};

pub fn generate_info(
    arch: &Arch,
    if_id: &pipes::IfPipe,
    id_ex: &pipes::IdPipe,
    ex_mem: &pipes::ExPipe,
    mem_wb: &pipes::MemPipe,
) -> info::PipelineDetail {
    info::PipelineDetail {
        debug_ins_if: Some(try_disassembly(arch, arch.pc())),
        debug_ins_id: if_id.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_ex: id_ex.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_mem: ex_mem.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_wb: mem_wb.debug_pc.map(|x| try_disassembly(arch, x)),
        nodes: vec![],
    }
}

fn try_disassembly(arch: &Arch, pc: u32) -> String {
    let ins = arch.mem.read_u32(pc);
    disassemble(ins)
}
