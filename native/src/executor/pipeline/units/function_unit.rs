pub fn funct_unit(_funct: u32, _alu_a: u32, _alu_b: u32, _shamt: u32) -> u32 {
    match _funct {
        0b100000 => _alu_a + _alu_b,    //add
        0b100010 => _alu_a - _alu_b,    //sub
        0b100100 => _alu_a & _alu_b,    //and
        0b100101 => _alu_a | _alu_b,    //or
        0b000000 => _alu_a << _shamt,   //sll
        0b000010 => _alu_a >> _shamt,   //srl
        0b100110 => _alu_a ^ _alu_b,     //xor
        0b100111 => !(_alu_a | _alu_b), //nor
        0b011000 => _alu_a * _alu_b,    //mult
        0b011001 => _alu_a / _alu_b,    //div
        _ => panic!("Invalid funct code"),
    }
}
