use crate::architecture::pipes;

pub fn ctrl_unit(opcode: u32) -> pipes::CtrUnit {
    match opcode {
        0b000000 => pipes::CtrUnit {
            // R-Type
            reg_dst: 0b1,
            reg_write: 0b1,
            alu_src: 0b0,
            alu_op: 0b10,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b0,
        },
        0b100011 => pipes::CtrUnit {
            // LW
            reg_dst: 0b0,
            reg_write: 0b1,
            alu_src: 0b1,
            alu_op: 0b00,
            mem_to_reg: 0b1,
            mem_read: 0b1,
            mem_write: 0b0,
            branch: 0b0,
        },
        0b101011 => pipes::CtrUnit {
            // SW
            reg_dst: 0b0,
            reg_write: 0b1,
            alu_src: 0b1,
            alu_op: 0b00,
            mem_to_reg: 0b1,
            mem_read: 0b1,
            mem_write: 0b0,
            branch: 0b0,
        },
        0b001000 => pipes::CtrUnit {
            // ADDI
            reg_dst: 0b0,
            reg_write: 0b1,
            alu_src: 0b1,
            alu_op: 0b00,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b0,
        },
        0b000100 => pipes::CtrUnit {
            // BEQ
            reg_dst: 0b0,
            reg_write: 0b0,
            alu_src: 0b0,
            alu_op: 0b01,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b1,
        },
        0b000101 => pipes::CtrUnit {
            // BNE
            reg_dst: 0b0,
            reg_write: 0b0,
            alu_src: 0b0,
            alu_op: 0b01,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b1,
        },
        0b000010 => pipes::CtrUnit {
            // J
            reg_dst: 0b0,
            reg_write: 0b0,
            alu_src: 0b0,
            alu_op: 0b00,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b1,
        },
        0b000011 => pipes::CtrUnit {
            // JAL
            reg_dst: 0b0,
            reg_write: 0b0,
            alu_src: 0b0,
            alu_op: 0b00,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b1,
        },
        0b100000 => pipes::CtrUnit {
            // LB
            reg_dst: 0b0,
            reg_write: 0b1,
            alu_src: 0b0,
            alu_op: 0b10,
            mem_to_reg: 0b0,
            mem_read: 0b0,
            mem_write: 0b0,
            branch: 0b0,
        },
        _ => pipes::CtrUnit::default(),
    }
}
