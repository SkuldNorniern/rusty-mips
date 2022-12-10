#[derive(Copy, Clone)]
pub struct FwdUnit {
    pub fwd_a: u32,
    pub fwd_b: u32,
    pub if_id_write: bool,
    pub pc_write: bool,
    pub hazard: bool,
}
impl Default for FwdUnit {
    fn default() -> FwdUnit {
        FwdUnit {
            fwd_a: 0x0,
            fwd_b: 0x0,
            if_id_write: true,
            pc_write: true,
            hazard: false,
        }
    }
}
