use crate::memory::endian_mode::EndianMode;
use std::collections::HashMap;
use std::ops::RangeInclusive;

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

    #[allow(clippy::reversed_empty_ranges)]
    pub fn address_range(&self) -> RangeInclusive<u32> {
        if !self.data.is_empty() {
            self.base_addr..=(self.base_addr + self.data.len() as u32)
        } else {
            1..=0
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
