use crate::memory::endian_mode::EndianMode;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Segment {
    pub base_addr: u32,
    pub data: Vec<u8>,

    // Offset starting from 0. Does not consider base_addr into calculation
    labels: HashMap<String, u32>,
    endian: EndianMode,
}

impl Segment {
    pub fn new(base_addr: u32, endian: EndianMode) -> Self {
        Segment {
            base_addr,
            labels: HashMap::new(),
            data: vec![],
            endian,
        }
    }

    pub fn overlaps_with(&self, other: &Segment) -> bool {
        let my_base = self.base_addr as u64;
        let other_base = other.base_addr as u64;

        if my_base <= other_base {
            other_base < my_base + (self.data.len() as u64)
        } else {
            my_base < other_base + (other.data.len() as u64)
        }
    }

    pub fn labels(&self) -> &HashMap<String, u32> {
        &self.labels
    }

    pub fn next_address(&self) -> u32 {
        self.base_addr + self.data.len() as u32
    }

    pub fn append_u32(&mut self, data: u32) {
        let mut buf = [0; 4];
        self.endian.write_u32(&mut buf, data);
        self.data.extend_from_slice(&buf);
    }

    pub fn append_label(&mut self, name: impl Into<String>) {
        self.labels.insert(name.into(), self.data.len() as u32);
    }
}
