use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::{forward_unit, function_unit};

pub fn next(_id_ex: &mut pipes::IdPipe, _fwd_unit: forward_unit::FwdUnit) -> pipes::ExPipe {
    let mut ex_mem = pipes::ExPipe {
        ran: _id_ex.ran,
        ..Default::default()
    };

    ex_mem.ctr_unit.mem_to_reg = _id_ex.ctr_unit.mem_to_reg;
    ex_mem.ctr_unit.mem_read = _id_ex.ctr_unit.mem_read;
    ex_mem.ctr_unit.mem_write = _id_ex.ctr_unit.mem_write;
    ex_mem.ctr_unit.branch = _id_ex.ctr_unit.branch;
    ex_mem.ctr_unit.reg_write = _id_ex.ctr_unit.reg_write;

    ex_mem.branch_tgt = _id_ex.npc + (_id_ex.imm << 2);

    let alu_a = _fwd_unit.fwd_a;
    let mut alu_b = _fwd_unit.fwd_b;
    if _id_ex.ctr_unit.alu_src == 0 {
        alu_b = _id_ex.imm;
    }

    if alu_a == alu_b {
        ex_mem.zero = 1;
    } else {
        ex_mem.zero = 0;
    }

    if _id_ex.ctr_unit.alu_op == 0 {
        ex_mem.alu_out = alu_a + alu_b;
    } else if _id_ex.ctr_unit.alu_op == 1 {
        ex_mem.alu_out = alu_a - alu_b;
    } else if _id_ex.ctr_unit.alu_op == 2 {
        let funct = _id_ex.imm & 0x0000003F;
        let shamt = _id_ex.imm & 0x000007C0;
        ex_mem.alu_out = function_unit::funct_unit(funct, alu_a, alu_b, shamt);
    }
    ex_mem.data_b = _fwd_unit.fwd_b;

    if _id_ex.ctr_unit.reg_dst == 1 {
        ex_mem.rd = _id_ex.rd;
    } else {
        ex_mem.rd = _id_ex.rt;
    }

    ex_mem
}
