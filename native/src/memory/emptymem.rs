use std::ptr::NonNull;
use crate::memory::{EndianMode, Memory};

// Something like /dev/null

#[derive(Debug)]
pub struct EmptyMem {
    endian: EndianMode,
}

impl EmptyMem {
    pub fn new(endian: EndianMode) -> Box<EmptyMem> {
        Box::new(EmptyMem {
            endian
        })
    }
}

impl Memory for EmptyMem {
    fn endian(&self) -> EndianMode {
        self.endian
    }

    fn fastmem_addr(&self) -> Option<NonNull<u8>> {
        None
    }

    fn read_u8(&self, _addr: u32) -> u8 { 0 }

    fn read_u16(&self, _addr: u32) -> u16 { 0 }

    fn read_u32(&self, _addr: u32) -> u32 { 0 }

    fn write_u8(&mut self, _addr: u32, _data: u8) {}

    fn write_u16(&mut self, _addr: u32, _data: u16) {}

    fn write_u32(&mut self, _addr: u32, _data: u32) {}

    fn write_from_slice(&mut self, _addr: u32, _data: &[u8]) {}
}
