use crate::memory::slowmem::SlowMem;
use crate::memory::{EndianMode, Segment};

pub trait Memory {
    fn endian(&self) -> EndianMode;

    fn read_u8(&self, addr: u32) -> u8;
    fn read_u16(&self, addr: u32) -> u16;
    fn read_u32(&self, addr: u32) -> u32;

    fn write_u8(&mut self, addr: u32, data: u8);
    fn write_u16(&mut self, addr: u32, data: u16);
    fn write_u32(&mut self, addr: u32, data: u32);

    fn write_from_slice(&mut self, addr: u32, data: &[u8]);
}

pub fn create_memory(endian: EndianMode, segments: &[Segment]) -> Box<dyn Memory> {
    SlowMem::new(endian, segments)
}
