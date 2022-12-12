use super::{EndianMode, Memory, Segment};
use lazy_static::lazy_static;
use lockfree::map::Map;
use std::alloc::Layout;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, EXCEPTION_ACCESS_VIOLATION, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, RemoveVectoredExceptionHandler, EXCEPTION_POINTERS,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile3, UnmapViewOfFile2, VirtualAlloc2, VirtualFree,
    MEM_PRESERVE_PLACEHOLDER, MEM_RELEASE, MEM_REPLACE_PLACEHOLDER, MEM_RESERVE,
    MEM_RESERVE_PLACEHOLDER, PAGE_NOACCESS, PAGE_READWRITE, VIRTUAL_FREE_TYPE,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::{s, w};

lazy_static! {
    static ref MEMORY_LIST: Map<usize, MemoryListEntry> = Map::new();
}

struct MemoryListEntry(*const FastMemWindows);
unsafe impl Send for MemoryListEntry {}
unsafe impl Sync for MemoryListEntry {}

pub struct FastMemWindows {
    base_addr: *mut u8,
    curr_process: HANDLE,
    handler: *mut c_void,
    pagefiles: [AtomicIsize; 1048576],
}

unsafe impl Send for FastMemWindows {}
unsafe impl Sync for FastMemWindows {}

impl Debug for FastMemWindows {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastMemWindows")
            .field("base_addr", &self.base_addr)
            .finish()
    }
}

impl FastMemWindows {
    pub fn try_new(endian: EndianMode, segments: &[Segment]) -> Option<Box<Self>> {
        if endian != EndianMode::native() || !has_virtualalloc2() {
            return None;
        }

        let mut this = unsafe {
            let curr_process = GetCurrentProcess();

            let base_addr = reserve_4gb(curr_process);
            if base_addr.is_null() {
                return None;
            }

            let ptr =
                std::alloc::alloc_zeroed(Layout::new::<FastMemWindows>()) as *mut FastMemWindows;

            // Init fields
            (&mut (*ptr).base_addr as *mut *mut u8).write(base_addr);
            (&mut (*ptr).curr_process as *mut HANDLE).write(curr_process);
            (&mut (*ptr).handler as *mut *mut c_void).write(null_mut());
            (&mut (*ptr).pagefiles as *mut AtomicIsize).write_bytes(0xFF, 1048576);
            // Done

            MEMORY_LIST.insert(base_addr as usize, MemoryListEntry(ptr));
            let handler: *mut c_void = AddVectoredExceptionHandler(1, Some(exception_handler));
            if handler.is_null() {
                // Failed to add handler
                MEMORY_LIST.remove(&(base_addr as usize));
                drop(Box::from_raw(ptr));
                return None;
            }

            // Put the handler
            (&mut (*ptr).handler as *mut *mut c_void).write(handler);

            Box::from_raw(ptr)
        };

        for seg in segments {
            this.write_from_slice(seg.base_addr, &seg.data);
        }

        Some(this)
    }
}

unsafe extern "system" fn exception_handler(info: *mut EXCEPTION_POINTERS) -> i32 {
    if (*(*info).ExceptionRecord).ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        let op: usize = (*(*info).ExceptionRecord).ExceptionInformation[0];
        let addr: usize = (*(*info).ExceptionRecord).ExceptionInformation[1];

        if op != 0 && op != 1 {
            // neither read nor write
            return 0;
        }

        for x in MEMORY_LIST.iter() {
            let base_addr = *x.key();
            let obj = x.val().0;

            if addr < base_addr || base_addr + 0xffff_ffff < addr {
                // address out of range
                continue;
            }

            let host_page = addr & (!0xfff);
            let page_idx = (addr - base_addr) / 4096;

            for _ in 0..3 {
                if (*obj).pagefiles[page_idx].load(Ordering::SeqCst) != -1 {
                    // looks like already loaded
                    // but if we return -1 here, this could result in an infinite loop
                    //TODO: Check if memory region is accessible, and return accordingly
                    return 0;
                }

                if let Ok(x) = allocate_page(GetCurrentProcess(), host_page as *mut c_void) {
                    if (*obj).pagefiles[page_idx]
                        .compare_exchange(-1, x.0, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        return -1;
                    }
                }
            }

            // tried twice and couldn't allocate memory region
            return 0;
        }

        // did not find any appropriate memory region
        return 0;
    }

    // unrelated error
    0
}

impl Memory for FastMemWindows {
    fn endian(&self) -> EndianMode {
        EndianMode::native()
    }

    fn fastmem_addr(&self) -> Option<NonNull<u8>> {
        NonNull::new(self.base_addr)
    }

    fn read_u8(&self, addr: u32) -> u8 {
        unsafe { (self.base_addr.add(addr as usize) as *mut u8).read_unaligned() }
    }

    fn read_u16(&self, addr: u32) -> u16 {
        unsafe { (self.base_addr.add(addr as usize) as *mut u16).read_unaligned() }
    }

    fn read_u32(&self, addr: u32) -> u32 {
        unsafe { (self.base_addr.add(addr as usize) as *mut u32).read_unaligned() }
    }

    fn write_u8(&mut self, addr: u32, data: u8) {
        unsafe { (self.base_addr.add(addr as usize) as *mut u8).write_unaligned(data) }
    }

    fn write_u16(&mut self, addr: u32, data: u16) {
        unsafe { (self.base_addr.add(addr as usize) as *mut u16).write_unaligned(data) }
    }

    fn write_u32(&mut self, addr: u32, data: u32) {
        unsafe { (self.base_addr.add(addr as usize) as *mut u32).write_unaligned(data) }
    }

    fn write_from_slice(&mut self, addr: u32, data: &[u8]) {
        assert!(data.len() <= u32::MAX as usize, "data too long");
        assert!(
            addr <= u32::MAX - data.len() as u32,
            "cannot write past memory"
        );

        unsafe {
            self.base_addr
                .add(addr as usize)
                .copy_from(data.as_ptr(), data.len());
        }
    }
}

impl Drop for FastMemWindows {
    fn drop(&mut self) {
        unsafe {
            // Remove signal handler
            let status: u32 = RemoveVectoredExceptionHandler(self.handler);
            self.handler = null_mut();
            if status == 0 {
                panic!(
                    "failed on RemoveVectoredExceptionHandler, GetLastError()={:?}",
                    GetLastError()
                );
            }

            // Deallocate all pages
            for (i, pagefile) in self.pagefiles.iter().enumerate() {
                let handle = pagefile.swap(-1, Ordering::AcqRel);
                if handle != INVALID_HANDLE_VALUE.0 {
                    let _ = deallocate_page(
                        self.curr_process,
                        self.base_addr.add(i * 4096),
                        HANDLE(handle),
                    );
                }
            }

            // Free the placeholder
            let _ = unreserve_4gb(self.base_addr);
            self.base_addr = null_mut();
        }
    }
}

fn has_virtualalloc2() -> bool {
    unsafe {
        // Modules acquired this way should not be freed

        match GetModuleHandleW(w!("kernel32")) {
            Ok(module) => {
                if GetProcAddress(module, s!("VirtualAlloc2")).is_some() {
                    return true;
                }
            }
            Err(_) => return false,
        };

        match GetModuleHandleW(w!("KernelBase")) {
            Ok(module) => {
                if GetProcAddress(module, s!("VirtualAlloc2")).is_some() {
                    return true;
                }
            }
            Err(_) => return false,
        };
    }

    false
}

/**
Allocate 4GB of placeholder + 4kb of guard page.
Placeholders don't count as used memory, so this won't consume 4gb of memory.
This will fail on 32-bits system.
 */
unsafe fn reserve_4gb(curr_process: HANDLE) -> *mut u8 {
    if std::mem::size_of::<usize>() <= std::mem::size_of::<u32>() {
        return null_mut();
    }

    VirtualAlloc2(
        curr_process,
        None,
        u32::MAX as usize + 4096,
        MEM_RESERVE | MEM_RESERVE_PLACEHOLDER,
        PAGE_NOACCESS.0,
        None,
    ) as *mut u8
}

unsafe fn unreserve_4gb(base_addr: *mut u8) -> Result<(), ()> {
    if VirtualFree(base_addr as *mut c_void, 0, MEM_RELEASE).as_bool() {
        Ok(())
    } else {
        Err(())
    }
}

unsafe fn allocate_page(process: HANDLE, page: *mut c_void) -> Result<HANDLE, ()> {
    // Don't check the result because it fails if the placeholder is already split to right size
    VirtualFree(
        page,
        4096,
        VIRTUAL_FREE_TYPE(MEM_RELEASE.0 | MEM_PRESERVE_PLACEHOLDER.0),
    );

    let size: u64 = 4096;
    let size_hi = (size >> 32) as u32;
    let size_lo = size as u32;

    let pagefile = CreateFileMappingW(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        size_hi,
        size_lo,
        PCWSTR(null_mut()),
    )
    .map_err(|_| ())?;

    let status = MapViewOfFile3(
        pagefile,
        process,
        Some(page),
        0,
        4096,
        MEM_REPLACE_PLACEHOLDER,
        PAGE_READWRITE.0,
        None,
    );

    if status.is_null() {
        return Err(());
    }

    Ok(pagefile)
}

unsafe fn deallocate_page(process: HANDLE, page: *mut u8, pagefile: HANDLE) -> Result<(), ()> {
    let status = UnmapViewOfFile2(process, page as *mut c_void, MEM_PRESERVE_PLACEHOLDER);
    if !status.as_bool() {
        return Err(());
    }

    let status = CloseHandle(pagefile);
    if !status.as_bool() {
        return Err(());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use parking_lot::Mutex;

    lazy_static! {
        // Lock for prevent tests from running concurrently
        static ref TEST_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn basic() {
        let _guard = TEST_LOCK.lock();

        let mut mem = FastMemWindows::try_new(EndianMode::native(), &[]).expect("creating memory");

        mem.write_u32(0x12345678, 4321);
        assert_eq!(mem.read_u32(0x12345678), 4321);

        assert_ne!(mem.pagefiles[0x12345678 / 4096].load(Ordering::SeqCst), -1);
    }

    #[test]
    fn multiple_instance() {
        let _guard = TEST_LOCK.lock();

        let mut mem1 = FastMemWindows::try_new(EndianMode::native(), &[]).expect("creating memory");
        let mut mem2 = FastMemWindows::try_new(EndianMode::native(), &[]).expect("creating memory");

        mem1.write_u32(0x1234, 1);
        mem2.write_u32(0x4321, 2);
        assert_eq!(mem1.read_u32(0x1234), 1);
        assert_eq!(mem2.read_u32(0x4321), 2);

        // check if corresponding pages are allocated
        assert_ne!(mem1.pagefiles[1].load(Ordering::SeqCst), -1);
        assert_ne!(mem2.pagefiles[4].load(Ordering::SeqCst), -1);

        drop(mem1);
        drop(mem2);
    }
}
