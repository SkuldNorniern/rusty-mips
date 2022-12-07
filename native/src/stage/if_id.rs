use crate::architecture::pipes;
use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};

pub fn next(if_id: &mut pipes::if_pipe) -> pipes::id_pipe {
    let mut id_ex: pipes::id_pipe = pipes::id_pipe::default();

    //let mut rdrins = Cursor::new(if_id.get("INST").unwrap());
    //println!("if_id: {:?}", if_id);
    //println!("pc: {:?}", rdrpc);

    //instruction = rdrins.read_u32::<NativeEndian>().unwrap();

    id_ex.npc = if_id.npc;

    //id_ex.append(&mut if_id[4..8].to_vec());
    //println!("id_ex: {:?}", id_ex);
    id_ex
}
