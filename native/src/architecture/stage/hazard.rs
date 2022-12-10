use crate::architecture::pipes;
use crate::architecture::units::forward_unit;

pub fn hazard_ctrl(_if_id: pipes::IfPipe,_id_ex: pipes::IdPipe , _fwd_unit: forward_unit::FwdUnit) -> forward_unit::FwdUnit {
    let mut fwd_unit = _fwd_unit;

    let if_id_rs = (_if_id.inst & 0x03E00000) >> 21;
    let if_id_rt = (_if_id.inst & 0x001F0000) >> 16;
    
    if _id_ex.ctr_unit.reg_dst == 1 && (_id_ex.rt == if_id_rs || _id_ex.rt == if_id_rt) {
        fwd_unit.hazard = true;
        fwd_unit.pc_write = false;
        fwd_unit.if_id_write = false;
    } else if _id_ex.ctr_unit.reg_dst == 0 && (_id_ex.rd == if_id_rs || _id_ex.rd == if_id_rt) {
        fwd_unit.if_id_write = false;
        fwd_unit.hazard = true;
    } else {
        fwd_unit.pc_write = true;
        fwd_unit.if_id_write = true;
        fwd_unit.hazard = false;
        
    }
        
    fwd_unit
}
