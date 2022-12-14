use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::forward_unit;

pub fn if_next(
    _inst: u32,
    _fwd: forward_unit::FwdUnit,
    _ex_mem: &mut pipes::ExPipe,
    _pc: u32,
    _finalize: bool
) -> (pipes::IfPipe, u32) {
    let mut if_id = pipes::IfPipe::default();
    let mut pc = _pc;
    if_id.ran = _pc;

    if _fwd.if_id_write {
        if_id.npc = _pc + 4;
        if_id.inst = _inst;
    }

    if _fwd.pc_write && !_finalize{
        if _ex_mem.zero == 1 && _ex_mem.ctr_unit.branch == 1 {
            pc = _ex_mem.branch_tgt;
        } else if !_fwd.hazard {
            pc = _pc + 4;
        }
    }
    (if_id, pc)
}
