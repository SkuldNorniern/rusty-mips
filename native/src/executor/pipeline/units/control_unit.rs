use crate::executor::pipeline::pipes;

pub fn ctrl_unit(opcode: u32) -> pipes::CtrUnitFull {
    match opcode {
        0b000000 => pipes::CtrUnitFull {
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
        0b100011 => pipes::CtrUnitFull {
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
        0b101011 => pipes::CtrUnitFull {
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
        0b001000 => pipes::CtrUnitFull {
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
        0b000100 => pipes::CtrUnitFull {
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
        0b000101 => pipes::CtrUnitFull {
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
        0b000010 => pipes::CtrUnitFull {
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
        0b000011 => pipes::CtrUnitFull {
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
        0b100000 => pipes::CtrUnitFull {
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
        _ => pipes::CtrUnitFull::default(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ctrl_unit() {
        use super::*;
        let test = ctrl_unit(0b000000);
        assert_eq!(test.reg_dst, 0b1);
        assert_eq!(test.reg_write, 0b1);
        assert_eq!(test.alu_src, 0b0);
        assert_eq!(test.alu_op, 0b10);
        assert_eq!(test.mem_to_reg, 0b0);
        assert_eq!(test.mem_read, 0b0);
        assert_eq!(test.mem_write, 0b0);
        assert_eq!(test.branch, 0b0);
    }
}
