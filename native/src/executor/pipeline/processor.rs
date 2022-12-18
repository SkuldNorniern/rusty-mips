use super::pipes;
use super::stage;
use super::units;
use crate::component::{Instruction, RegisterName};
use crate::executor::Arch;
use crate::memory::Memory;

use crate::disassembler::disassemble;
use crate::executor::pipeline::stage::ex_stage::ex_next;
use crate::executor::pipeline::stage::id_stage::id_next;
use crate::executor::pipeline::stage::if_stage::if_next;
use crate::executor::pipeline::stage::mem_stage::mem_next;
use crate::executor::pipeline::stage::wb_stage::wb_next;
use crate::executor::pipeline::units::hazard_unit::hazard_ctrl;
use std::collections::HashMap;
use crate::executor::pipeline::{debug, info};

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
}

impl Pipeline {
    pub fn new(mem: Box<dyn Memory>) -> Pipeline {
        Pipeline {
            arch: Arch::new(mem),
            if_id: pipes::IfPipe::default(),
            id_ex: pipes::IdPipe::default(),
            ex_mem: pipes::ExPipe::default(),
            mem_wb: pipes::MemPipe::default(),
            wb: pipes::WbPipe::default(),
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

    pub fn next(&mut self, finalize: bool) {
        cfg_if::cfg_if! {
            if #[cfg(test)] {
                const LOG_OUTPUT: bool = true;
            } else {
                const LOG_OUTPUT: bool = false;
            }
        }

        if LOG_OUTPUT {
            println!("PROC: step: {:08x}", self.arch.pc());
        }

        let fetch_pc = self.as_arch().pc();
        let fetch_ins = self.arch.mem.read_u32(fetch_pc);

        let if_output = if_next(fetch_pc, fetch_ins, finalize);
        let id_output = id_next(&self.if_id, &self.arch, &self.ex_mem, &self.mem_wb);
        let ex_output = ex_next(&self.id_ex, &self.arch, &self.ex_mem, &self.mem_wb);
        let mem_output = mem_next(&self.ex_mem, &mut self.arch);
        let wb_output = wb_next(&self.mem_wb, &mut self.arch);

        let hazard = hazard_ctrl(&if_output, &id_output, &ex_output, &mem_output);

        let next_pc = if id_output.branch_taken {
            id_output.branch_target
        } else if id_output.jump_taken {
            id_output.jump_target
        } else {
            self.arch.pc().wrapping_add(4)
        };

        if LOG_OUTPUT {
            let read_disassemble = |debug_pc: Option<u32>| {
                disassemble(self.arch.mem.read_u32(debug_pc.unwrap_or(0)))
            };

            println!("IF ins: {}", disassemble(if_output.inst));
            println!("IF: {:?}", if_output);
            println!(
                "ID ins: {}",
                read_disassemble(id_output.debug_pc)
            );
            println!("ID: {:?}", id_output);
            println!(
                "EX ins: {}",
                read_disassemble(ex_output.debug_pc)
            );
            println!("EX: {:?}", ex_output);
            println!(
                "MEM ins: {}",
                read_disassemble(mem_output.debug_pc)
            );
            println!("MEM: {:?}", mem_output);
            println!(
                "WB ins: {}",
                read_disassemble(wb_output.debug_pc)
            );
            println!("WB: {:?}", wb_output);
            println!("Hazard: {:?}", hazard);
            println!("Next PC: {:08x}", next_pc);
            println!();
        }

        self.mem_wb = mem_output;
        self.ex_mem = ex_output;
        self.id_ex = id_output;

        if !hazard.redo_if && !hazard.drop_if {
            self.if_id = if_output;
        } else {
            self.if_id = Default::default();
        }

        if !hazard.redo_if && !finalize {
            // advance PC if we're not re-fetching current instruction
            self.arch.set_pc(next_pc);
        }
    }

    pub fn step(&mut self) {
        self.next(false);
    }

    pub fn finalize(&mut self) {
        self.next(true);
    }

    pub fn get_pipeline_detail(&self) -> info::PipelineDetail {
        debug::generate_info(&self.arch, &self.if_id, &self.id_ex, &self.ex_mem, &self.mem_wb)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assembler::assemble;
    use crate::disassembler::disassemble;
    use crate::executor::Interpreter;
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

    #[test]
    fn bubblesort() {
        let code = r"
.data 0x10008000
.word 5 1 4 2 3

.text
.globl main
main:
    or      $a0, $0, $gp
    addi    $a1, $0, 5
    j       bubblesort

bubblesort:  # prototype: void bubblesort(int* arr, int len)
    addi    $15, $0, 1
    slt     $15, $5, $15
    beq     $15, $0, .L8

    j       .L1
.L8:
    addi    $2, $0, 1
    beq     $5, $2, .L1

    sll     $7, $5, 2
    add     $7, $4, $7
    or      $9, $0, $0
    addi    $10, $4, 4
.L6:
    or      $2, $10, $0
    or      $8, $0, $0
.L4:
    lw      $3, -4($2)
    lw      $4, 0($2)
    slt     $6, $4, $3
    beq     $6, $0, .L3

    sw      $4, -4($2)
    sw      $3, 0($2)
    addi    $8, $0, 1
.L3:
    addi    $2, $2, 4
    beq     $7, $2, .L7

    j       .L4
.L7:
    beq     $8, $0, .L1

    addi    $9, $9, 1
    beq     $5, $9, .L1

    j       .L6

.L1:
    j       0x00000000";

        let segs = assemble(EndianMode::native(), code).unwrap();
        let mut interpreter = Interpreter::new(create_memory(EndianMode::native(), &segs));

        let mut expected = vec![];
        while interpreter.as_arch().pc() != 0 {
            expected.push(interpreter.as_arch().pc());
            interpreter.step().unwrap();
        }

        drop(interpreter);
        let mut proc = make(code);

        for expected_pc in &expected {
            proc.step();

            // skip over hazard bubbles
            for _ in 0..5 {
                if proc.id_ex.debug_pc.is_some() {
                    break;
                }
                proc.step();
            }

            // hazard must have been skipped, or it is some kind of bug
            let pipeline_pc= proc.id_ex.debug_pc.unwrap();
            assert_eq!(pipeline_pc, *expected_pc, "{:08x} vs {:08x}", pipeline_pc, expected_pc);
        }

        assert_eq!(proc.arch.pc(), 0);
    }
}
