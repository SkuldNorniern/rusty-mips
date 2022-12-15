use super::pipes;
use super::stage;
use super::units;
use crate::component::RegisterName;
use crate::executor::Arch;
use crate::memory::Memory;

use std::collections::HashMap;

pub struct Description {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Pipeline {
    arch: Arch,
    if_id: pipes::IfPipe,
    id_ex: pipes::IdPipe,
    ex_mem: pipes::ExPipe,
    mem_wb: pipes::MemPipe,
    wb: pipes::WbPipe,
    fwd_unit: units::forward_unit::FwdUnit,
}

impl Pipeline {
    pub fn new(mem: Box<dyn Memory>) -> Pipeline {
        Pipeline {
            arch: Arch::new(mem),
            if_id: { pipes::IfPipe::default() },
            id_ex: { pipes::IdPipe::default() },
            ex_mem: { pipes::ExPipe::default() },
            mem_wb: { pipes::MemPipe::default() },
            wb: { pipes::WbPipe::default() },
            fwd_unit: { units::forward_unit::FwdUnit::default() },
        }
    }

    pub fn as_arch(&self) -> &Arch {
        &self.arch
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        &mut self.arch
    }

    pub fn into_arch(mut self) -> Arch {
        self.finalize();
        self.arch
    }

    pub fn reg(&self, name: u32) -> u32 {
        assert!(name < 32, "register name must be under 32");
        self.arch.reg(RegisterName::new(name as u8))
    }

    pub fn set_reg(&mut self, name: u32, val: u32) {
        assert!(name < 32, "register name must be under 32");
        self.arch.set_reg(RegisterName::new(name as u8), val);
    }

    pub fn next(&mut self, _finalize: bool) {
        println!("PROC: step: {}", self.arch.pc());
        self.fwd_unit = stage::forward::fwd_ctrl(
            &mut self.ex_mem,
            &mut self.mem_wb,
            &mut self.id_ex,
            self.fwd_unit,
        );
        println!(
            "WB  : memtoreg: {}, regwrite: {}",
            self.mem_wb.ctr_unit.mem_to_reg, self.mem_wb.ctr_unit.reg_write
        );
        let wb_tup = stage::wb_stage::next(&mut self.mem_wb);
        self.wb = wb_tup.0;
        println!("WB  : addr: {}, data: {}", wb_tup.1 .0, wb_tup.1 .1);
        self.set_reg(wb_tup.1 .0, wb_tup.1 .1);
        println!("REG : addr 18 data {}", self.reg(18));
        let lmd_addr = self.ex_mem.alu_out;

        let lmd = self.arch.mem.read_u32(lmd_addr);
        println!("MEM : lmd_addr: {}", lmd_addr);
        println!("MEM : lmd: {}", lmd);
        println!("MEM : ex_rd: {}", self.ex_mem.rd);
        let mem_tup = stage::mem_stage::next(&mut self.ex_mem, lmd);
        self.mem_wb = mem_tup.0;
        println!("MEM : addr : {}, data: {}", mem_tup.1 .0, mem_tup.1 .1);
        if mem_tup.1 .2 {
            self.arch.mem.write_u32(mem_tup.1 .0, mem_tup.1 .1);
        }

        self.ex_mem = stage::ex_stage::next(&mut self.id_ex, self.fwd_unit);

        let data_a_addr = (self.if_id.inst & 0x03E00000) >> 21;
        let data_a = self.reg(data_a_addr);
        let data_b_addr = (self.if_id.inst & 0x001F0000) >> 16;
        let data_b = self.reg(data_b_addr);
        println!("ID  : data_a: {}, data_b: {}", data_a, data_b);
        println!("ID  : id_rd: {}", self.id_ex.rd);
        println!("ID  : id_rs: {}", self.id_ex.rs);
        println!("ID  : id_rt: {}", self.id_ex.rt);
        self.id_ex =
            stage::id_stage::id_next(&mut self.if_id, self.fwd_unit.hazard, data_a, data_b);
        let mut cur_inst = self.arch.mem.read_u32(self.arch.pc());
        if _finalize {
            cur_inst = 0x00000000;
        }
        println!("{:?}", self.fwd_unit);
        self.fwd_unit = stage::hazard::hazard_ctrl(
            &mut self.if_id,
            &mut self.id_ex,
            &mut self.ex_mem,
            self.fwd_unit,
        );
        let if_tup = stage::if_stage::if_next(
            cur_inst,
            self.fwd_unit,
            &mut self.id_ex,
            &mut self.ex_mem,
            self.arch.pc(),
            _finalize,
        );
        println!("IF  : inst: {}", self.if_id.inst);
        self.if_id = if_tup.0;
        self.arch.set_pc(if_tup.1);
    }

    pub fn step(&mut self) {
        self.next(false);
    }

    pub fn finalize(&mut self) {
        self.next(true);
    }

    pub(crate) fn get_pipeline_detail(&self) -> HashMap<String, Description> {
        let mut map = HashMap::new();

        // IF/ID
        map.insert(
            "debug-if-pc".to_string(),
            Description {
                id: "debug-if-pc".to_string(),
                name: "Current PC".to_string(),
                value: format!("{:08x}", self.if_id.ran),
            },
        );
        map.insert(
            "svg-item-ifid-npc".to_string(),
            Description {
                id: "svg_item_if_id_npc".to_string(),
                name: "Next PC".to_string(),
                value: format!("0x{:08x}", self.if_id.npc),
            },
        );
        map.insert(
            "svg-item-ifid-inst".to_string(),
            Description {
                id: "svg_item_if_id_inst".to_string(),
                name: "Instruction".to_string(),
                value: format!("0x{:08x}", self.if_id.inst),
            },
        );

        // ID/EX
        map.insert(
            "debug-id-pc".to_string(),
            Description {
                id: "debug-id-pc".to_string(),
                name: "Current PC".to_string(),
                value: format!("{:08x}", self.id_ex.ran),
            },
        );
        map.insert(
            "svg-item-idex-npc".to_string(),
            Description {
                id: "svg_item_id_ex_npc".to_string(),
                name: "Next PC".to_string(),
                value: format!("0x{:08x}", self.id_ex.npc),
            },
        );
        map.insert(
            "svg-item-idex-a".to_string(),
            Description {
                id: "svg_item_id_ex_data_a".to_string(),
                name: "Data A".to_string(),
                value: format!("0x{:08x}", self.id_ex.data_a),
            },
        );
        map.insert(
            "svg-item-idex-b".to_string(),
            Description {
                id: "svg_item_id_ex_data_b".to_string(),
                name: "Data B".to_string(),
                value: format!("0x{:08x}", self.id_ex.data_b),
            },
        );
        map.insert(
            "svg-item-idex-imm".to_string(),
            Description {
                id: "svg_item_id_ex_imm".to_string(),
                name: "Immediate".to_string(),
                value: format!("0x{:08x}", self.id_ex.imm),
            },
        );
        map.insert(
            "svg-item-idex-rs".to_string(),
            Description {
                id: "svg_item_id_ex_rs".to_string(),
                name: "RS".to_string(),
                value: format!("0x{:08x}", self.id_ex.rs),
            },
        );
        map.insert(
            "svg-item-idex-rt".to_string(),
            Description {
                id: "svg_item_id_ex_rt".to_string(),
                name: "RT".to_string(),
                value: format!("0x{:08x}", self.id_ex.rt),
            },
        );
        map.insert(
            "svg-item-idex-rd".to_string(),
            Description {
                id: "svg_item_id_ex_rd".to_string(),
                name: "RD".to_string(),
                value: format!("0x{:08x}", self.id_ex.rd),
            },
        );

        // ID/EX Control
        map.insert(
            "debug-ex-pc".to_string(),
            Description {
                id: "debug-ex-pc".to_string(),
                name: "Current PC".to_string(),
                value: format!("{:08x}", self.ex_mem.ran),
            },
        );
        map.insert(
            "svg-item-idex-regdst".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_reg_dst".to_string(),
                name: "RegDst".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.reg_dst),
            },
        );
        map.insert(
            "svg-item-idex-regwrite".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_reg_write".to_string(),
                name: "RegWrite".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.reg_write),
            },
        );
        map.insert(
            "svg-item-idex-memread".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_mem_read".to_string(),
                name: "MemRead".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.mem_read),
            },
        );
        map.insert(
            "svg-item-idex-memwrite".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_mem_write".to_string(),
                name: "MemWrite".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.mem_write),
            },
        );
        map.insert(
            "svg-item-idex-memtoreg".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_mem_to_reg".to_string(),
                name: "MemToReg".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.mem_to_reg),
            },
        );
        map.insert(
            "svg-item-idex-aluop".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_alu_op".to_string(),
                name: "ALUOp".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.alu_op),
            },
        );
        map.insert(
            "svg-item-idex-alusrc".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_alu_src".to_string(),
                name: "ALUSrc".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.alu_src),
            },
        );
        map.insert(
            "svg-item-idex-branch".to_string(),
            Description {
                id: "svg_item_id_ex_ctrl_branch".to_string(),
                name: "Branch".to_string(),
                value: format!("{}", self.id_ex.ctr_unit.branch),
            },
        );

        // EX/MEM
        map.insert(
            "debug-mem-pc".to_string(),
            Description {
                id: "debug-mem-pc".to_string(),
                name: "Current PC".to_string(),
                value: format!("{:08x}", self.mem_wb.ran),
            },
        );
        map.insert(
            "svg-item-exmem-brtgt".to_string(),
            Description {
                id: "svg_item_ex_mem_br_tgt".to_string(),
                name: "Branch Target".to_string(),
                value: format!("0x{:08x}", self.ex_mem.branch_tgt),
            },
        );
        map.insert(
            "svg-item-exmem-zero".to_string(),
            Description {
                id: "svg_item_ex_mem_zero".to_string(),
                name: "Zero".to_string(),
                value: format!("0x{:08x}", self.ex_mem.zero),
            },
        );
        map.insert(
            "svg-item-exmem-aluout".to_string(),
            Description {
                id: "svg_item_ex_mem_alu_out".to_string(),
                name: "ALU Out".to_string(),
                value: format!("0x{:08x}", self.ex_mem.alu_out),
            },
        );
        map.insert(
            "svg-item-exmem-b".to_string(),
            Description {
                id: "svg_item_ex_mem_data_b".to_string(),
                name: "Data B".to_string(),
                value: format!("0x{:08x}", self.ex_mem.data_b),
            },
        );
        map.insert(
            "svg-item-exmem-rd".to_string(),
            Description {
                id: "svg_item_ex_mem_rd".to_string(),
                name: "RD".to_string(),
                value: format!("0x{:08x}", self.ex_mem.rd),
            },
        );

        // EX/MEM Control
        map.insert(
            "svg-item-exmem-regwrite".to_string(),
            Description {
                id: "svg_item_ex_mem_ctrl_reg_write".to_string(),
                name: "RegWrite".to_string(),
                value: format!("{}", self.ex_mem.ctr_unit.reg_write),
            },
        );
        map.insert(
            "svg-item-exmem-memread".to_string(),
            Description {
                id: "svg_item_ex_mem_ctrl_mem_read".to_string(),
                name: "MemRead".to_string(),
                value: format!("{}", self.ex_mem.ctr_unit.mem_read),
            },
        );
        map.insert(
            "svg-item-exmem-memwrite".to_string(),
            Description {
                id: "svg_item_ex_mem_ctrl_mem_write".to_string(),
                name: "MemWrite".to_string(),
                value: format!("{}", self.ex_mem.ctr_unit.mem_write),
            },
        );
        map.insert(
            "svg-item-exmem-memtoreg".to_string(),
            Description {
                id: "svg_item_ex_mem_ctrl_mem_to_reg".to_string(),
                name: "MemToReg".to_string(),
                value: format!("{}", self.ex_mem.ctr_unit.mem_to_reg),
            },
        );
        map.insert(
            "svg-item-exmem-branch".to_string(),
            Description {
                id: "svg_item_ex_mem_ctrl_branch".to_string(),
                name: "Branch".to_string(),
                value: format!("{}", self.ex_mem.ctr_unit.branch),
            },
        );

        // MEM/WB
        map.insert(
            "debug-wb-pc".to_string(),
            Description {
                id: "debug-wb-pc".to_string(),
                name: "Current PC".to_string(),
                value: format!("{:08x}", self.wb.ran),
            },
        );
        map.insert(
            "svg-item-memwb-lmd".to_string(),
            Description {
                id: "svg_item_mem_wb_lmd".to_string(),
                name: "LMD".to_string(),
                value: format!("0x{:08x}", self.mem_wb.lmd),
            },
        );
        map.insert(
            "svg-item-memwb-aluout".to_string(),
            Description {
                id: "svg_item_mem_wb_alu_out".to_string(),
                name: "ALU Out".to_string(),
                value: format!("0x{:08x}", self.mem_wb.alu_out),
            },
        );
        map.insert(
            "svg-item-memwb-rd".to_string(),
            Description {
                id: "svg_item_mem_wb_rd".to_string(),
                name: "RD".to_string(),
                value: format!("0x{:08x}", self.mem_wb.rd),
            },
        );

        // MEM/WB Control
        map.insert(
            "svg-item-memwb-regwrite".to_string(),
            Description {
                id: "svg_item_mem_wb_ctrl_reg_write".to_string(),
                name: "RegWrite".to_string(),
                value: format!("{}", self.mem_wb.ctr_unit.reg_write),
            },
        );
        map.insert(
            "svg-item-memwb-memtoreg".to_string(),
            Description {
                id: "svg_item_mem_wb_ctrl_mem_to_reg".to_string(),
                name: "MemToReg".to_string(),
                value: format!("{}", self.mem_wb.ctr_unit.mem_to_reg),
            },
        );

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    use crate::memory::{create_memory, EndianMode};

    fn make(asm: &str) -> Pipeline {
        let native = EndianMode::native();
        let segs = assemble(native, asm).unwrap();
        let mem = create_memory(native, &segs);
        Pipeline::new(mem)
    }

    #[test]
    fn processor_pc_value() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.arch.pc(), 0x00400028);
    }

    #[test]
    fn processor_cycle_1_inst() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
    }

    #[test]
    fn processor_cycle_2_inst() {
        let mut proc = make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();

        assert_eq!(proc.if_id.inst, 0x02001022);
    }

    #[test]
    fn processor_cycle_3_inst() {
        let mut proc = make(".text\nadd $18, $16, $17\nsub $2, $s0, $zero");
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02119020);
        proc.step();
        assert_eq!(proc.if_id.inst, 0x02001022);
        proc.step();
    }

    //TODO Add more tests
    #[test]
    fn stage_id_control_unit() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        let test: u32 = 0x02114020;
        let sample = units::control_unit::ctrl_unit((test & 0xFC000000) >> 26);

        assert_eq!(proc.id_ex.ctr_unit.reg_dst, sample.reg_dst);
        assert_eq!(proc.id_ex.ctr_unit.branch, sample.branch);
        assert_eq!(proc.id_ex.ctr_unit.mem_read, sample.mem_read);
        assert_eq!(proc.id_ex.ctr_unit.mem_to_reg, sample.mem_to_reg);
        assert_eq!(proc.id_ex.ctr_unit.alu_op, sample.alu_op);
        assert_eq!(proc.id_ex.ctr_unit.mem_write, sample.mem_write);
        assert_eq!(proc.id_ex.ctr_unit.alu_src, sample.alu_src);
        assert_eq!(proc.id_ex.ctr_unit.reg_write, sample.reg_write);
    }

    #[test]
    fn stage_id_datas() {
        let mut proc = make(".text\nadd $18, $16, $17");
        proc.step();
        assert_eq!(proc.id_ex.data_a, 0x0);
        assert_eq!(proc.id_ex.data_b, 0x0);
    }

    #[test]
    fn inst_addi() {
        let mut proc = make(".text\naddi $18, $0, 0x1\nnop\nnop\nnop\nnop\nnop\nnop");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(18), 0x1);
    }

    #[test]
    fn inst_lw() {
        let mut proc = make(
            ".data 0x10008000\n.word 1\n.text\nlw $18, 0x0($gp)\nnop\nnop\nnop\nnop\nnop\nnop",
        );
        proc.step();
        proc.step();
        println!("MEM::::{:?}", proc.arch.mem.read_u32(0x10008000));
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(18), 0x1);
    }
    #[test]
    fn inst_sw_lw() {
        let mut proc = make(".text\naddi $18, $0, 0x1\nsw $18, 0($gp)\nlw $19, 0($gp)\nnop\nnop\nnop\nnop\nnop\nnop");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(19), 0x1);
        //panic!()
    }
    #[test]
    fn inst_add() {
        let mut proc = make(".text\naddi $t0, $0, 1\naddi $t1, $0, 2\nadd $t2, $t1, $t0");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(8), 0x1);
        proc.step();
        assert_eq!(proc.reg(9), 0x2);
        proc.step();
        assert_eq!(proc.reg(10), 0x3);
    }
    #[test]
    fn many_nop() {
        let mut proc = make(".text\nnop\nnop\nnop\nnop");
        assert_eq!(proc.arch.pc(), 0x00400024);
        proc.step();
        assert_eq!(proc.arch.pc(), 0x00400028);
        proc.step();
        assert_eq!(proc.arch.pc(), 0x0040002c);
        proc.step();
        assert_eq!(proc.arch.pc(), 0x00400030);
        proc.step();
        assert_eq!(proc.arch.pc(), 0x00400034);
    }

    #[test]
    fn inst_sw() {
        let mut proc = make(".text\naddi $t0, $0, 1\nnop\nnop\nnop\nnop\nsw $t0, 0($gp)\nnop\nnop\nnop\nnop\nnop\nnop");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(8), 0x1);
        assert_eq!(proc.arch.mem.read_u32(0x10008000), 0x1);
    }
    #[test]
    fn inst_beq() {
        let mut proc = make(".text\na:\naddi $s0, $0, 1\nbeq $0, $0, b\naddi $s1, $0, 1\naddi $s2, $0, 1\nb:\naddi $s3, $0, 1");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(16), 0x1);
        assert_eq!(proc.reg(17), 0x0);
        assert_eq!(proc.reg(18), 0x0);
        assert_eq!(proc.reg(19), 0x1);
    }
    #[test]
    fn load_use() {
        let mut proc = make(".data 0x10008000\n.word 1\n.text\n.globl main\nmain:\naddi $s1, $0,1\nlw $s0, 0($gp)\nadd $s2, $s0, $s1");
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        proc.step();
        assert_eq!(proc.reg(17), 0x1);
        assert_eq!(proc.reg(18), 0x2);
    }
}
