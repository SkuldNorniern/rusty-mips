use bitflags::bitflags;

bitflags! {
    #[must_use]
    pub struct Updates: u16 {
        const REGISTERS = 1 << 0;
        const DISASSEMBLY = 1 << 2;
        const FLAG_RUNNING = 1 << 3;
        const FLAG_CAN_USE_JIT = 1 << 3;
    }
}
