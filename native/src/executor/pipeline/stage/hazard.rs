use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::forward_unit;

pub fn hazard_ctrl(
    _if_id: &mut pipes::IfPipe,
    _id_ex: &mut pipes::IdPipe,
    _ex_mem: &mut pipes::ExPipe,
    _fwd_unit: forward_unit::FwdUnit,
) -> forward_unit::FwdUnit {
    let mut fwd_unit = _fwd_unit;

    let if_id_rs = (_if_id.inst & 0x03E00000) >> 21;
    let if_id_rt = (_if_id.inst & 0x001F0000) >> 16;

    if _id_ex.ctr_unit.mem_read == 1 && (_id_ex.rt == if_id_rs || _id_ex.rt == if_id_rt) {
        fwd_unit.hazard = true;
        fwd_unit.pc_write = false;
        fwd_unit.if_id_write = false;
    } else if _id_ex.ctr_unit.branch == 1 || _ex_mem.ctr_unit.branch == 1 {
        fwd_unit.if_id_write = false;
        fwd_unit.hazard = true;
    } else {
        fwd_unit.pc_write = true;
        fwd_unit.if_id_write = true;
        fwd_unit.hazard = false;
    }

    fwd_unit
}
