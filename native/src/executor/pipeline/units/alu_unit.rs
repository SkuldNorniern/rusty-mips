pub enum AluOp {
    Add,
    And,
    Or,
    Sub,
    Slt,
    Sll,
}

pub fn alu_unit(a: u32, b: u32, shamt: u32, op: AluOp) -> u32 {
    match op {
        AluOp::Add => a.wrapping_add(b),
        AluOp::And => a & b,
        AluOp::Or => a | b,
        AluOp::Sub => a.wrapping_sub(b),
        AluOp::Slt => {
            if a < b {
                1
            } else {
                0
            }
        }
        AluOp::Sll => b << shamt,
    }
}
