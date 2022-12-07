use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::Cursor;

use crate::architecture::pipes;

pub fn next(_id_ex: &mut pipes::IdPipe) -> pipes::ExPipe {
    let mut ex_mem: pipes::ExPipe = pipes::ExPipe::default();
    //let mut branch_target: Vec<u8> = Vec::new();
    //let mut npc: Vec<u8> = Vec::new();
    //println!("if_id: {:?}", if_id);
    //println!("pc: {:?}", rdrpc);

    //id_ex.append(&mut if_id[4..8].to_vec());
    //println!("id_ex: {:?}", id_ex);
    ex_mem
}
