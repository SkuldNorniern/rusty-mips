use crate::component::{Instruction, TypeI, TypeJ, TypeR};

fn format_type_r(mnemonic: &str, x: TypeR) -> String {
    format!(
        "{} ${}, ${}, ${}",
        mnemonic,
        x.rd.name(),
        x.rs.name(),
        x.rt.name()
    )
}

fn format_type_shift(mnemonic: &str, x: TypeR) -> String {
    format!(
        "{} ${}, ${}, {}",
        mnemonic,
        x.rd.name(),
        x.rt.name(),
        x.shamt
    )
}

fn format_type_i(mnemonic: &str, x: TypeI) -> String {
    format!("{} ${}, ${}, {}", mnemonic, x.rt.name(), x.rs.name(), x.imm)
}

fn format_type_branch_2arg(mnemonic: &str, x: TypeI) -> String {
    format!(
        "{} ${}, ${}, {}",
        mnemonic,
        x.rt.name(),
        x.rs.name(),
        x.imm as u32 * 4
    )
}

fn format_type_branch_1arg(mnemonic: &str, x: TypeI) -> String {
    format!("{} ${}, {}", mnemonic, x.rt.name(), x.imm as u32 * 4)
}

fn format_type_memory(mnemonic: &str, x: TypeI) -> String {
    format!(
        "{} ${}, {}(${})",
        mnemonic,
        x.rt.name(),
        x.imm as i16,
        x.rs.name()
    )
}

fn format_type_jump_imm(mnemonic: &str, x: TypeJ) -> String {
    format!("{} 0x{:08x}", mnemonic, x.target * 4)
}

fn format_type_jump_reg(mnemonic: &str, x: TypeR) -> String {
    format!("{} ${}", mnemonic, x.rs.name())
}

fn format_type_jump_reg_linked(mnemonic: &str, x: TypeR) -> String {
    format!("{} ${}, ${}", mnemonic, x.rd.name(), x.rs.name())
}

pub fn disassemble(ins: u32) -> String {
    if ins == 0 {
        return "nop".into();
    }

    let decoded = Instruction::decode(ins);

    match decoded {
        Instruction::add(x) => format_type_r("add", x),
        Instruction::addu(x) => format_type_r("addu", x),
        Instruction::and(x) => format_type_r("and", x),
        Instruction::nor(x) => format_type_r("nor", x),
        Instruction::or(x) => format_type_r("or", x),
        Instruction::slt(x) => format_type_r("slt", x),
        Instruction::sltu(x) => format_type_r("sltu", x),
        Instruction::sub(x) => format_type_r("sub", x),
        Instruction::subu(x) => format_type_r("subu", x),
        Instruction::xor(x) => format_type_r("xor", x),
        Instruction::sll(x) => format_type_shift("sll", x),
        Instruction::sllv(x) => format_type_r("sllv", x),
        Instruction::sra(x) => format_type_shift("sra", x),
        Instruction::srav(x) => format_type_r("srav", x),
        Instruction::srl(x) => format_type_shift("srl", x),
        Instruction::srlv(x) => format_type_r("srlv", x),
        Instruction::addi(x) => format_type_i("addi", x),
        Instruction::addiu(x) => format_type_i("addiu", x),
        Instruction::andi(x) => format_type_i("andi", x),
        Instruction::lui(x) => format!("lui ${}, {}", x.rt.name(), x.imm),
        Instruction::ori(x) => format_type_i("ori", x),
        Instruction::slti(x) => format_type_i("slti", x),
        Instruction::sltiu(x) => format_type_i("sltiu", x),
        Instruction::xori(x) => format_type_i("xori", x),
        Instruction::beq(x) => format_type_branch_2arg("beq", x),
        Instruction::bgez(x) => format_type_branch_1arg("bgez", x),
        Instruction::bgezal(x) => format_type_branch_1arg("bgezal", x),
        Instruction::bgtz(x) => format_type_branch_1arg("bgtz", x),
        Instruction::blez(x) => format_type_branch_1arg("blez", x),
        Instruction::bltz(x) => format_type_branch_1arg("bltz", x),
        Instruction::bltzal(x) => format_type_branch_1arg("bltzal", x),
        Instruction::bne(x) => format_type_branch_2arg("bne", x),
        Instruction::lb(x) => format_type_memory("lb", x),
        Instruction::lbu(x) => format_type_memory("lbu", x),
        Instruction::lh(x) => format_type_memory("lh", x),
        Instruction::lhu(x) => format_type_memory("lhu", x),
        Instruction::lw(x) => format_type_memory("lw", x),
        Instruction::sb(x) => format_type_memory("sb", x),
        Instruction::sh(x) => format_type_memory("sh", x),
        Instruction::sw(x) => format_type_memory("sw", x),
        Instruction::j(x) => format_type_jump_imm("j", x),
        Instruction::jal(x) => format_type_jump_imm("jal", x),
        Instruction::jalr(x) => format_type_jump_reg_linked("jalr", x),
        Instruction::jr(x) => format_type_jump_reg("jr", x),
        Instruction::syscall(_) => "syscall".into(),
        Instruction::invalid(_) => format!("<invalid instruction 0x{:08x}>", ins),
    }
}
