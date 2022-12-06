use std::collections::HashMap;

pub struct Segment {
    // Offset starting from 0. Does not consider base_addr into calculation
    pub labels: HashMap<String, u32>,
    pub data: Vec<u8>,
    pub base_addr: u32,
}

impl Segment {
    pub fn new(base_addr: u32) -> Self {
        Segment {
            labels: HashMap::new(),
            data: vec![],
            base_addr,
        }
    }
}
