use crate::architecture::pipes;

pub fn if_next(_inst: u32, _is_hazard: bool, _cur_addr: u32) -> pipes::IfPipe {
    let mut if_id = pipes::IfPipe::default();
    if_id.ran = _cur_addr;

    if _is_hazard {
        if_id.inst = 0x00000000;
    } else {
        if_id.inst = _inst;
    }
    if_id.npc = _cur_addr + 4;

    if_id
}
