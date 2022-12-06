use std::collections::HashMap;
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
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

    #[allow(clippy::reversed_empty_ranges)]
    pub fn address_range(&self) -> RangeInclusive<u32> {
        if !self.data.is_empty() {
            self.base_addr..=(self.base_addr + self.data.len() as u32)
        } else {
            1..=0
        }
    }
}
