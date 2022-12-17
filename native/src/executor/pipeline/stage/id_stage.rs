use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::{control_unit, forward_unit};
use crate::executor::Arch;
use std::process::id;

pub fn id_next(
    id_input: &pipes::IfPipe,
    arch: &Arch,
    mem_input: &pipes::ExPipe,
    wb_input: &pipes::MemPipe,
) -> pipes::IdPipe {
    let opcode = (id_input.inst & 0xFC000000) >> 26;
    let rs = (id_input.inst & 0x03E00000) >> 21;
    let rt = (id_input.inst & 0x001F0000) >> 16;
    let rd = (id_input.inst & 0x0000F800) >> 11;
    let imm = id_input.inst as u16 as i16 as i32 as u32;

    let jump_target = (id_input.npc & 0xF0000000) | ((id_input.inst & 0x03FFFFFF) << 2);
    let jump_taken = opcode == 2 || opcode == 3;

    let branch_target = (imm << 2).wrapping_add(id_input.npc);
    let ctr_unit = control_unit::ctrl_unit(opcode);
    let data_a = forward_unit::forward_value(rs, arch, mem_input, wb_input);
    let data_b = forward_unit::forward_value(rt, arch, mem_input, wb_input);
    let branch_taken = ctr_unit.branch && data_a == data_b;

    pipes::IdPipe {
        npc: id_input.npc,
        data_a,
        data_b,
        opcode,
        rs,
        rt,
        rd,
        imm,
        ctr_unit,
        branch_target,
        branch_taken,
        jump_target,
        jump_taken,
        debug_pc: id_input.debug_pc,
    }
}
