use crate::disassembler::disassemble;
use crate::executor::pipeline::{info, pipes};
use crate::executor::Arch;

pub fn generate_info(
    arch: &Arch,
    if_id: &pipes::IfPipe,
    id_ex: &pipes::IdPipe,
    ex_mem: &pipes::ExPipe,
    mem_wb: &pipes::MemPipe,
) -> info::PipelineDetail {
    use info::PipelineNodeInfo::*;

    let nodes = vec![
        Hex32 {
            id: "if_id_next_pc",
            name: "Next PC",
            value: if_id.npc,
        },
        Hex32 {
            id: "if_id_instruction",
            name: "Instruction",
            value: if_id.inst,
        },
        Bool {
            id: "id_ex_memtoreg",
            name: "Instruction",
            value: id_ex.ctr_unit.mem_to_reg,
        },
        Bool {
            id: "id_ex_memtoreg",
            name: "Memory to Register",
            value: id_ex.ctr_unit.mem_to_reg,
        },
        Bool {
            id: "id_ex_memwrite",
            name: "Write Memory",
            value: id_ex.ctr_unit.mem_write,
        },
        Bool {
            id: "id_ex_memread",
            name: "Read Memory",
            value: id_ex.ctr_unit.mem_read,
        },
        Bool {
            id: "id_ex_branch",
            name: "Branch",
            value: id_ex.ctr_unit.branch,
        },
        Bool {
            id: "id_ex_regwrite",
            name: "Write Register",
            value: id_ex.ctr_unit.reg_write,
        },
        Bool {
            id: "id_ex_regdst",
            name: "Desination Register",
            value: id_ex.ctr_unit.reg_dst,
        },
        Bool {
            id: "id_ex_alusrc",
            name: "ALU Source",
            value: id_ex.ctr_unit.alu_src,
        },
        Dec {
            id: "id_ex_aluop",
            name: "ALU Operation",
            value: id_ex.ctr_unit.alu_op as _,
        },
        Bool {
            id: "id_ex_jump",
            name: "Jump",
            value: id_ex.ctr_unit.jump,
        },
        Bool {
            id: "id_ex_branch_taken",
            name: "Jump",
            value: id_ex.ctr_unit.branch,
        },
        Hex32 {
            id: "id_ex_next_pc",
            name: "Next PC",
            value: id_ex.npc,
        },
        Hex32 {
            id: "id_ex_branch_target",
            name: "Branch Target",
            value: id_ex.branch_target,
        },
        Hex32 {
            id: "id_ex_jump_target",
            name: "Jump Target",
            value: id_ex.jump_target,
        },
        Hex32 {
            id: "id_ex_data_a",
            name: "Data A",
            value: id_ex.data_a,
        },
        Hex32 {
            id: "id_ex_data_b",
            name: "Data B",
            value: id_ex.data_b,
        },
        Dec {
            id: "id_ex_rt",
            name: "rt",
            value: id_ex.rt as _,
        },
        Dec {
            id: "id_ex_rd",
            name: "rd",
            value: id_ex.rd as _,
        },
        Dec {
            id: "id_ex_imm",
            name: "Immediate",
            value: id_ex.imm as i32 as i64,
        },
        Dec {
            id: "id_ex_rs",
            name: "rs",
            value: id_ex.rs as _,
        },
        Bool {
            id: "ex_mem_memtoreg",
            name: "Memory to Register",
            value: ex_mem.ctr_unit.mem_to_reg,
        },
        Bool {
            id: "ex_mem_memwrite",
            name: "Write Memory",
            value: ex_mem.ctr_unit.mem_write,
        },
        Bool {
            id: "ex_mem_memread",
            name: "Read Memory",
            value: ex_mem.ctr_unit.mem_read,
        },
        Bool {
            id: "ex_mem_regwrite",
            name: "Write Register",
            value: ex_mem.ctr_unit.mem_read,
        },
        Hex32 {
            id: "ex_mem_aluout",
            name: "ALU Output",
            value: ex_mem.alu_out,
        },
        Hex32 {
            id: "ex_mem_b",
            name: "Data B",
            value: ex_mem.data_b,
        },
        Dec {
            id: "ex_mem_rd",
            name: "rd",
            value: ex_mem.rd as _,
        },
        Bool {
            id: "mem_wb_memtoreg",
            name: "Memory to Register",
            value: mem_wb.ctr_unit.mem_to_reg,
        },
        Bool {
            id: "mem_wb_regwrite",
            name: "Write Register",
            value: mem_wb.ctr_unit.reg_write,
        },
        Hex32 {
            id: "mem_wb_lmd",
            name: "Loaded Memory Data",
            value: mem_wb.lmd,
        },
        Hex32 {
            id: "mem_wb_aluout",
            name: "ALU Output",
            value: mem_wb.alu_out,
        },
        Dec {
            id: "mem_wb_rd",
            name: "rd",
            value: mem_wb.rd as _,
        },
    ];

    info::PipelineDetail {
        debug_ins_if: Some(try_disassembly(arch, arch.pc())),
        debug_ins_id: if_id.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_ex: id_ex.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_mem: ex_mem.debug_pc.map(|x| try_disassembly(arch, x)),
        debug_ins_wb: mem_wb.debug_pc.map(|x| try_disassembly(arch, x)),
        nodes,
    }
}

fn try_disassembly(arch: &Arch, pc: u32) -> String {
    let ins = arch.mem.read_u32(pc);
    disassemble(ins)
}
