use crate::executor::pipeline::pipes;

pub fn if_next(pc: u32, inst: u32, finalize: bool) -> pipes::IfPipe {
    let output_inst = if !finalize { inst } else { 0 };

    pipes::IfPipe {
        npc: pc.wrapping_add(4),
        inst: output_inst,
        debug_pc: Some(pc),
    }
}
