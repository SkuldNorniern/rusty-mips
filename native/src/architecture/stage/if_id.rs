use crate::architecture::pipes;


pub fn next(_if_id: &mut pipes::IfPipe) -> pipes::IdPipe {
    let mut id_ex: pipes::IdPipe = pipes::IdPipe::default();

    //let mut rdrins = Cursor::new(if_id.get("INST").unwrap());
    //println!("if_id: {:?}", if_id);
    //println!("pc: {:?}", rdrpc);

    //instruction = rdrins.read_u32::<NativeEndian>().unwrap();

    id_ex.npc = _if_id.npc;

    //id_ex.append(&mut if_id[4..8].to_vec());
    //println!("id_ex: {:?}", id_ex);
    id_ex
}
