pub mod ex_mem;
pub mod id_ex;
pub mod if_id;
pub mod mem_wb;

use crate::architecture::pipes;

pub fn if_id_stage(_if_id: &mut pipes::IfPipe) -> pipes::IdPipe {
    
    if_id::next(_if_id)
}
pub fn id_ex_stage() {
    //id_ex::next();
}
pub fn ex_mem_stage() {
    //ex_mem::next();
}
pub fn mem_wb_stage() {
    //mem_wb::next();
}
