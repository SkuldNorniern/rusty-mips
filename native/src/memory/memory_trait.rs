use crate::memory::{EndianMode, Segment};
use std::fmt::Debug;
use std::ptr::NonNull;

pub trait Memory: Send + Sync + Debug {
    fn endian(&self) -> EndianMode;

    // If this memory type is fastmem-compatible, return the base address
    fn fastmem_addr(&self) -> Option<NonNull<u8>>;

    fn read_u8(&self, addr: u32) -> u8;
    fn read_u16(&self, addr: u32) -> u16;
    fn read_u32(&self, addr: u32) -> u32;

    fn write_u8(&mut self, addr: u32, data: u8);
    fn write_u16(&mut self, addr: u32, data: u16);
    fn write_u32(&mut self, addr: u32, data: u32);

    fn write_from_slice(&mut self, addr: u32, data: &[u8]);
}

pub fn create_memory(endian: EndianMode, segments: &[Segment]) -> Box<dyn Memory> {
    use super::slowmem::SlowMem;

    cfg_if::cfg_if! {
        if #[cfg(test)] {
            SlowMem::new(endian, segments)
        } else if #[cfg(windows)] {
            super::fastmem_windows::FastMemWindows::try_new(endian, segments)
                .map(|x| -> Box<dyn Memory> { x })
                .unwrap_or_else(|| SlowMem::new(endian, segments))
        } else if #[cfg(unix)] {
            super::fastmem_unix::FastMemUnix::try_new(endian, segments)
                .map(|x| -> Box<dyn Memory> { x })
                .unwrap_or_else(|| SlowMem::new(endian, segments))
        } else {
            SlowMem::new(endian, segments)
        }
    }
}

pub fn create_empty_memory(endian: EndianMode) -> Box<dyn Memory> {
    super::emptymem::EmptyMem::new(endian)
}
