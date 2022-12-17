use crate::executor::pipeline::pipes;
use crate::executor::pipeline::units::alu_unit::AluOp;
use crate::executor::pipeline::units::{alu_unit, forward_unit};
use crate::executor::Arch;

pub fn ex_next(
    ex_input: &pipes::IdPipe,
    arch: &Arch,
    mem_input: &pipes::ExPipe,
    wb_input: &pipes::MemPipe,
) -> pipes::ExPipe {
    let data_a = forward_unit::forward_value(ex_input.rs, arch, mem_input, wb_input);
    let data_b = forward_unit::forward_value(ex_input.rt, arch, mem_input, wb_input);

    let alu_out = match ex_input.opcode {
        0 => {
            let shamt = (ex_input.imm & 0x000007C0) >> 6;
            let funct = ex_input.imm & 0x0000003F;
            let op = match funct {
                0x00 => AluOp::Sll,
                0x20 | 0x21 => AluOp::Add,
                0x22 | 0x23 => AluOp::Sub,
                0x24 => AluOp::And,
                0x25 => AluOp::Or,
                0x2a => AluOp::Slt,
                _ => {
                    println!("unknown funct {}", funct);
                    AluOp::Add
                }
            };
            alu_unit::alu_unit(data_a, data_b, shamt, op)
        }
        0x8 | 0x23 | 0x2b => data_a.wrapping_add(ex_input.imm),
        _ => 0,
    };

    let rd = if ex_input.ctr_unit.reg_dst {
        ex_input.rd
    } else {
        ex_input.rt
    };

    pipes::ExPipe {
        alu_out,
        data_b,
        rd,
        ctr_unit: ex_input.ctr_unit.clone(),
        debug_pc: ex_input.debug_pc,
    }
}
