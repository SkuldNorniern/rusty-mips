use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::control_unit;

pub fn id_next(_if_id: &mut pipes::IfPipe, _is_hazard: bool, _reg_list: &[u32]) -> pipes::IdPipe {
    let mut id_ex = pipes::IdPipe {
        ran: _if_id.ran,
        ..Default::default()
    };

    if _is_hazard {
        id_ex.ctr_unit.reg_dst = 0b0;
        id_ex.ctr_unit.reg_write = 0b0;
        id_ex.ctr_unit.alu_src = 0b0;
        id_ex.ctr_unit.alu_op = 0b0;
        id_ex.ctr_unit.mem_to_reg = 0b0;
        id_ex.ctr_unit.mem_read = 0b0;
        id_ex.ctr_unit.mem_write = 0b0;
        id_ex.ctr_unit.branch = 0b0;
        id_ex.ctr_unit.if_flush = 0b0;
    } else {
        let opcode = _if_id.inst & 0xFC000000 >> 26;
        id_ex.ctr_unit = control_unit::ctrl_unit(opcode);
    }

    id_ex.npc = _if_id.npc;
    id_ex.data_a = _reg_list[((_if_id.inst & 0x03E00000) >> 21) as usize];
    id_ex.data_b = _reg_list[((_if_id.inst & 0x001F0000) >> 16) as usize];
    id_ex.rs = (_if_id.inst & 0x03E00000) >> 21;
    id_ex.rt = (_if_id.inst & 0x001F0000) >> 16;
    id_ex.rd = (_if_id.inst & 0x0000F800) >> 11;
    id_ex.imm = _if_id.inst & 0x0000FFFF;
    id_ex.ctr_unit.if_flush = 0b0;
    if id_ex.ctr_unit.branch == 1 && id_ex.data_a == id_ex.data_b {
        id_ex.ctr_unit.if_flush = 0b1;
    }

    id_ex
}
