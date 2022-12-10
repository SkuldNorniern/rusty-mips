pub fn funct_unit(_funct: u32, _alu_A: u32, _alu_b: u32, _shamt: u32) -> u32 {
    match _funct {
        0b100000 => _alu_A + _alu_b,  //add
        0b100010 => _alu_A - _alu_b,  //sub
        0b100100 => _alu_A & _alu_b,  //and
        0b100101 => _alu_A | _alu_b,  //or
        0b000000 => _alu_A << _shamt, //sll
        0b000010 => _alu_A >> _shamt, //srl
        0b011000 => _alu_A * _alu_b,  //mult
        0b011010 => _alu_A / _alu_b,  //div
        _ => panic!("Invalid funct code"),
    }
}
