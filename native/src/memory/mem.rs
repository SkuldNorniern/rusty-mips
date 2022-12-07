use super::segment::Segment;
use byteorder::{NativeEndian, ReadBytesExt};
use std::io::Cursor;
use std::iter::FromIterator;

pub struct Memory {
    read_only_segments: Vec<Segment>,
}

impl Memory {
    pub fn new(segments: &[Segment]) -> Self {
        Memory {
            read_only_segments: Vec::from_iter(segments.iter().cloned()),
        }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        for seg in &self.read_only_segments {
            if seg.address_range().contains(&addr) {
                let slice = &seg.data[(addr - seg.base_addr) as usize..];
                let mut cursor = Cursor::new(slice);
                return cursor.read_u32::<NativeEndian>().unwrap();
            }
        }

        0
    }

    pub fn write_u32(&self, addr: u32, data: u32) {}
}
