use crate::memory::memory_trait::Memory;
use crate::memory::{EndianMode, Segment};
use std::alloc::Layout;
use std::convert::TryInto;

const PAGE_SIZE: u32 = 4096;

type Page = [u8; 4096];
type Pages = [Option<Box<Page>>; 1048576];

fn create_boxed_page() -> Box<Page> {
    assert!(
        std::mem::size_of::<Page>() > 0,
        "Page must be not zero-sized"
    );

    unsafe {
        // SAFETY: Page is not zero-sized
        let ptr = std::alloc::alloc_zeroed(Layout::new::<Page>()) as *mut Page;

        // SAFETY: u8 of value 0 is valid
        Box::from_raw(ptr)
    }
}

pub struct SlowMem {
    endian: EndianMode,
    pages: Pages,
}

impl SlowMem {
    pub fn new(endian: EndianMode, segments: &[Segment]) -> Box<Self> {
        let mut obj = unsafe {
            // SAFETY: SlowMem is not zero-sized type
            let ptr = std::alloc::alloc_zeroed(Layout::new::<SlowMem>()) as *mut SlowMem;

            // SAFETY: ptr is allocated, and we're using `write`
            let endian_ptr = &mut (*ptr).endian as *mut EndianMode;
            endian_ptr.write(endian);

            // SAFETY: This is allocated just above, and we wrote to every field
            //         (except pages, which is an array full of None)
            Box::from_raw(ptr)
        };

        for seg in segments {
            obj.write_from_slice(seg.base_addr, &seg.data);
        }

        obj
    }

    #[inline(never)]
    fn read_u16_unaligned(&self, addr: u32) -> u16 {
        let b0 = self.read_u8(addr) as u16;
        let b1 = self.read_u8(addr + 1) as u16;
        match self.endian {
            EndianMode::Big => b0 << 8 | b1,
            EndianMode::Little => b1 << 8 | b0,
        }
    }

    #[inline(never)]
    fn read_u32_unaligned(&self, addr: u32) -> u32 {
        let b0 = self.read_u8(addr) as u32;
        let b1 = self.read_u8(addr + 1) as u32;
        let b2 = self.read_u8(addr + 2) as u32;
        let b3 = self.read_u8(addr + 3) as u32;
        match self.endian {
            EndianMode::Big => b0 << 24 | b1 << 16 | b2 << 8 | b3,
            EndianMode::Little => b3 << 24 | b2 << 16 | b1 << 8 | b0,
        }
    }

    #[inline(never)]
    fn write_u16_unaligned(&mut self, addr: u32, data: u16) {
        let mut buf = [0; 2];
        self.endian.write_u16(&mut buf, data);
        self.write_from_slice(addr, &buf);
    }

    #[inline(never)]
    fn write_u32_unaligned(&mut self, addr: u32, data: u32) {
        let mut buf = [0; 4];
        self.endian.write_u32(&mut buf, data);
        self.write_from_slice(addr, &buf);
    }

    #[inline(always)]
    fn ensure_page(&mut self, page_idx: usize) -> &mut Page {
        if self.pages[page_idx as usize].is_none() {
            self.pages[page_idx as usize] = Some(create_boxed_page());
        }
        self.pages[page_idx as usize].as_mut().unwrap()
    }
}

impl Memory for SlowMem {
    fn endian(&self) -> EndianMode {
        self.endian
    }

    fn read_u8(&self, addr: u32) -> u8 {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;

        self.pages[page_idx as usize]
            .as_ref()
            .map(|x| x[page_offset as usize])
            .unwrap_or(0)
    }

    fn read_u16(&self, addr: u32) -> u16 {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;
        if page_offset + 1 < PAGE_SIZE {
            self.pages[page_idx as usize]
                .as_ref()
                .map(|x| self.endian.read_u16(&x[page_offset as usize..]))
                .unwrap_or(0)
        } else {
            self.read_u16_unaligned(addr)
        }
    }

    fn read_u32(&self, addr: u32) -> u32 {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;
        if page_offset + 3 < PAGE_SIZE {
            self.pages[page_idx as usize]
                .as_ref()
                .map(|x| self.endian.read_u32(&x[page_offset as usize..]))
                .unwrap_or(0)
        } else {
            self.read_u32_unaligned(addr)
        }
    }

    fn write_u8(&mut self, addr: u32, data: u8) {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;

        self.ensure_page(page_idx as usize)[page_offset as usize] = data;
    }

    fn write_u16(&mut self, addr: u32, data: u16) {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;

        if page_offset + 1 < PAGE_SIZE {
            self.ensure_page(page_idx as usize);
            let page = self.pages[page_idx as usize]
                .as_mut()
                .expect("page already allocated");
            self.endian
                .write_u16(&mut page[page_offset as usize..], data);
        } else {
            self.write_u16_unaligned(addr, data);
        }
    }

    fn write_u32(&mut self, addr: u32, data: u32) {
        let page_idx = addr / PAGE_SIZE;
        let page_offset = addr % PAGE_SIZE;

        if page_offset + 3 < PAGE_SIZE {
            self.ensure_page(page_idx as usize);
            let page = self.pages[page_idx as usize]
                .as_mut()
                .expect("page already allocated");
            self.endian
                .write_u32(&mut page[page_offset as usize..], data);
        } else {
            self.write_u32_unaligned(addr, data);
        }
    }

    fn write_from_slice(&mut self, addr: u32, data: &[u8]) {
        assert!(
            TryInto::<u32>::try_into(data.len()).is_ok(),
            "data length must be less than u32"
        );

        if data.is_empty() {
            return;
        }

        let mut page_idx = (addr / PAGE_SIZE) as usize;
        let mut pos = 0;
        let mut page_offset = addr % PAGE_SIZE;
        loop {
            let remaining_bytes = (data.len() - pos) as u32;
            let writable_bytes = PAGE_SIZE - page_offset;
            let copy_bytes = u32::min(remaining_bytes, writable_bytes) as usize;

            let src_from = pos;
            let src_to = src_from + copy_bytes;
            let dst_from = page_offset as usize;
            let dst_to = dst_from + copy_bytes;

            self.ensure_page(page_idx)[dst_from..dst_to].copy_from_slice(&data[src_from..src_to]);

            page_offset = 0;
            pos += copy_bytes;
            page_idx += 1;

            if remaining_bytes <= writable_bytes {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let test = |endian| {
            let mut mem = SlowMem::new(endian, &[]);
            mem.write_u8(0, 42);
            assert_eq!(mem.read_u8(0), 42);
            mem.write_u16(0, 42);
            assert_eq!(mem.read_u16(0), 42);
            mem.write_u32(0, 42);
            assert_eq!(mem.read_u32(0), 42);
        };

        test(EndianMode::Little);
        test(EndianMode::Big);
    }

    #[test]
    fn read_uncommitted() {
        let test = |endian| {
            let mem = SlowMem::new(endian, &[]);
            assert_eq!(mem.read_u32(0x12345678), 0);
            assert_eq!(mem.read_u32(0), 0);
            assert_eq!(mem.read_u32(0xfffffff0), 0);
        };

        test(EndianMode::Little);
        test(EndianMode::Big);
    }

    #[test]
    fn unaligned() {
        let test = |endian| {
            let mut mem = SlowMem::new(endian, &[]);
            mem.write_u16(4095, 42);
            assert_eq!(mem.read_u16(4095), 42);
            mem.write_u32(4094, 0x12345678);
            assert_eq!(mem.read_u32(4094), 0x12345678);
            mem.write_u32(4095, 0x12345678);
            assert_eq!(mem.read_u32(4095), 0x12345678);
        };

        test(EndianMode::Little);
        test(EndianMode::Big);
    }

    #[test]
    fn endian() {
        let mut mem = SlowMem::new(EndianMode::Little, &[]);
        mem.write_u32(0, 0x11223344);
        assert_eq!(mem.read_u8(0), 0x44);
        assert_eq!(mem.read_u8(1), 0x33);
        assert_eq!(mem.read_u8(2), 0x22);
        assert_eq!(mem.read_u8(3), 0x11);

        let mut mem = SlowMem::new(EndianMode::Big, &[]);
        mem.write_u32(0, 0x11223344);
        assert_eq!(mem.read_u8(0), 0x11);
        assert_eq!(mem.read_u8(1), 0x22);
        assert_eq!(mem.read_u8(2), 0x33);
        assert_eq!(mem.read_u8(3), 0x44);
    }
}
