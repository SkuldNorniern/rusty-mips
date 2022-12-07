use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

pub fn next(if_id: &mut HashMap<String,Vec<u8>> ) -> HashMap<String, Vec<u8>> {
    let mut id_ex: HashMap<String, Vec<u8>> = HashMap::new();
    let mut pc: u32;
    let mut instruction: u32;
    let mut rdrpc = Cursor::new(if_id.get("NPC").unwrap());
    let mut rdrins = Cursor::new(if_id.get("INST").unwrap());
    //println!("if_id: {:?}", if_id);
    //println!("pc: {:?}", rdrpc);
    pc = rdrpc.read_u32::<NativeEndian>().unwrap();

    instruction = rdrins.read_u32::<NativeEndian>().unwrap();
    
    id_ex.insert(String::from("NPC"), if_id.get("NPC").unwrap().to_vec());
    //id_ex.append(&mut if_id[4..8].to_vec());
    //println!("id_ex: {:?}", id_ex);
    id_ex
}
