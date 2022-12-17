use crate::executor::pipeline::pipes;

pub fn ctrl_unit(opcode: u32) -> pipes::CtrFlags {
    match opcode {
        0b000000 => pipes::CtrFlags {
            // R-Type
            reg_dst: true,
            reg_write: true,
            alu_src: false,
            alu_op: 0b10,
            mem_to_reg: false,
            mem_read: false,
            mem_write: false,
            branch: false,
            jump: false,
            if_flush: false,
        },
        0b100011 => pipes::CtrFlags {
            // LW
            reg_dst: false,
            reg_write: true,
            alu_src: true,
            alu_op: 0b00,
            mem_to_reg: true,
            mem_read: true,
            mem_write: false,
            branch: false,
            jump: false,
            if_flush: false,
        },
        0b101011 => pipes::CtrFlags {
            // SW
            reg_dst: false,
            reg_write: false,
            alu_src: true,
            alu_op: 0b00,
            mem_to_reg: false,
            mem_read: false,
            mem_write: true,
            branch: false,
            jump: false,
            if_flush: false,
        },
        0b001000 => pipes::CtrFlags {
            // ADDI
            reg_dst: false,
            reg_write: true,
            alu_src: true,
            alu_op: 0b00,
            mem_to_reg: false,
            mem_read: false,
            mem_write: false,
            branch: false,
            jump: false,
            if_flush: false,
        },
        0b000100 => pipes::CtrFlags {
            // BEQ
            reg_dst: false,
            reg_write: false,
            alu_src: false,
            alu_op: 0b01,
            mem_to_reg: false,
            mem_read: false,
            mem_write: false,
            branch: true,
            jump: false,
            if_flush: false,
        },
        0b000010 => pipes::CtrFlags {
            // J
            reg_dst: false,
            reg_write: false,
            alu_src: false,
            alu_op: 0b11,
            mem_to_reg: false,
            mem_read: false,
            mem_write: false,
            branch: false,
            jump: true,
            if_flush: false,
        },
        _ => pipes::CtrFlags::default(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ctrl_unit() {
        use super::*;
        let test = ctrl_unit(0b000000);
        assert_eq!(test.reg_dst, true);
        assert_eq!(test.reg_write, true);
        assert_eq!(test.alu_src, false);
        assert_eq!(test.alu_op, 0b10);
        assert_eq!(test.mem_to_reg, false);
        assert_eq!(test.mem_read, false);
        assert_eq!(test.mem_write, false);
        assert_eq!(test.branch, false);
        assert_eq!(test.if_flush, false);
    }
}
