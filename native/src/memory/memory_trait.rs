use crate::memory::{EndianMode, Segment};
use std::fmt::Debug;
use std::ptr::{slice_from_raw_parts, NonNull};

pub trait Memory: Send + Sync + Debug {
    fn endian(&self) -> EndianMode;

    // If this memory type is fastmem-compatible, return the base address
    fn fastmem_addr(&self) -> Option<NonNull<u8>>;

    fn read_u8(&self, addr: u32) -> u8;
    fn read_u16(&self, addr: u32) -> u16;
    fn read_u32(&self, addr: u32) -> u32;

    fn read_into_slice(&self, addr: u32, output: &mut [u8]);

    fn write_u8(&mut self, addr: u32, data: u8);
    fn write_u16(&mut self, addr: u32, data: u16);
    fn write_u32(&mut self, addr: u32, data: u32);

    fn write_from_slice(&mut self, addr: u32, data: &[u8]);
}

pub trait FastMem: Send + Sync + Debug {
    fn fastmem_addr(&self) -> *mut u8;
}

impl<T> Memory for T
where
    T: FastMem,
{
    fn endian(&self) -> EndianMode {
        EndianMode::native()
    }

    fn fastmem_addr(&self) -> Option<NonNull<u8>> {
        NonNull::new(self.fastmem_addr())
    }

    fn read_u8(&self, addr: u32) -> u8 {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u8).read_unaligned() }
    }

    fn read_u16(&self, addr: u32) -> u16 {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u16).read_unaligned() }
    }

    fn read_u32(&self, addr: u32) -> u32 {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u32).read_unaligned() }
    }

    fn read_into_slice(&self, addr: u32, output: &mut [u8]) {
        assert!(output.len() <= u32::MAX as usize, "length too long");
        assert!(
            addr <= u32::MAX - output.len() as u32,
            "cannot read past memory"
        );

        unsafe {
            let src = slice_from_raw_parts(self.fastmem_addr().add(addr as usize), output.len());
            output.copy_from_slice(&*src);
        }
    }

    fn write_u8(&mut self, addr: u32, data: u8) {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u8).write_unaligned(data) }
    }

    fn write_u16(&mut self, addr: u32, data: u16) {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u16).write_unaligned(data) }
    }

    fn write_u32(&mut self, addr: u32, data: u32) {
        unsafe { (self.fastmem_addr().add(addr as usize) as *mut u32).write_unaligned(data) }
    }

    fn write_from_slice(&mut self, addr: u32, data: &[u8]) {
        assert!(data.len() <= u32::MAX as usize, "data too long");
        assert!(
            addr <= u32::MAX - data.len() as u32,
            "cannot write past memory"
        );

        unsafe {
            self.fastmem_addr()
                .add(addr as usize)
                .copy_from(data.as_ptr(), data.len());
        }
    }
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
