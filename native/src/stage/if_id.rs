use std::io::Cursor;
use byteorder::{ByteOrder, NativeEndian,ReadBytesExt,WriteBytesExt};


pub fn next(if_id: &mut Vec<u8>) -> Vec<u8>{
    
    let mut id_ex: Vec<u8> = Vec::new();
    let mut pc: u32 ;
    let mut instruction: u32;
    let mut rdrpc = Cursor::new(if_id[0..4].to_vec());
    let mut rdrins = Cursor::new(if_id[4..8].to_vec());
    //println!("if_id: {:?}", if_id);
    //println!("pc: {:?}", rdrpc);
    pc = rdrpc.read_u32::<NativeEndian>().unwrap();
    
    instruction = rdrins.read_u32::<NativeEndian>().unwrap();
    id_ex.append(&mut if_id[0..4].to_vec());
    //id_ex.append(&mut if_id[4..8].to_vec());
    //println!("id_ex: {:?}", id_ex);
    id_ex
}
