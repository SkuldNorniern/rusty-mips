#[allow(non_camel_case_types)]
pub enum Mnemonic {
    // R format
    add,
    and,
    or,
    sub,
    slt,

    // I format
    lw,
    sw,
    beq,

    // J format
    j,
}
