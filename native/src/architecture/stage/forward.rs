use crate::architecture::pipes;
use crate::architecture::units::forward_unit;

pub fn fwd_ctrl(
    _ex_mem: &mut pipes::ExPipe,
    _mem_wb: &mut pipes::MemPipe,
    _id_ex: &mut pipes::IdPipe,
    _fwd_unit: forward_unit::FwdUnit,
) -> forward_unit::FwdUnit {
    let mut fwd_unit = _fwd_unit;

    if _mem_wb.ctr_unit.mem_to_reg == 1
        && _mem_wb.rd != 0
        && _mem_wb.rd == _id_ex.rs
        && (_ex_mem.rd != _id_ex.rs || _ex_mem.ctr_unit.reg_write == 0)
    {
        fwd_unit.fwd_a = 1;
    } else if _mem_wb.ctr_unit.mem_to_reg == 1 && _mem_wb.rd != 0 && _mem_wb.rd == _id_ex.rs {
        fwd_unit.fwd_a = 2;
    } else {
        fwd_unit.fwd_a = 0;
    }

    if _mem_wb.ctr_unit.mem_to_reg == 1
        && _mem_wb.rd != 0
        && _mem_wb.rd == _id_ex.rt
        && (_ex_mem.rd != _id_ex.rt || _ex_mem.ctr_unit.reg_write == 0)
    {
        fwd_unit.fwd_b = 1;
    } else if _mem_wb.ctr_unit.mem_to_reg == 1 && _mem_wb.rd != 0 && _mem_wb.rd == _id_ex.rt {
        fwd_unit.fwd_b = 2;
    } else {
        fwd_unit.fwd_b = 0;
    }

    if _fwd_unit.fwd_a == 0 {
        fwd_unit.fwd_a = _id_ex.data_a;
    } else if _fwd_unit.fwd_a == 1 {
        if _mem_wb.ctr_unit.mem_to_reg == 1 {
            fwd_unit.fwd_a = _mem_wb.lmd;
        } else {
            fwd_unit.fwd_a = _mem_wb.alu_out;
        }
    } else if _fwd_unit.fwd_a == 2 {
        fwd_unit.fwd_a = _ex_mem.alu_out;
    }

    if _fwd_unit.fwd_b == 0 {
        fwd_unit.fwd_b = _id_ex.data_b;
    } else if _fwd_unit.fwd_b == 1 {
        if _mem_wb.ctr_unit.mem_to_reg == 1 {
            fwd_unit.fwd_b = _mem_wb.lmd;
        } else {
            fwd_unit.fwd_b = _mem_wb.alu_out;
        }
    } else if _fwd_unit.fwd_b == 2 {
        fwd_unit.fwd_b = _ex_mem.alu_out;
    }

    fwd_unit
}
