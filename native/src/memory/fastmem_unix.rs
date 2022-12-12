use super::memory_trait::FastMem;
use super::{EndianMode, Memory, Segment};
use lazy_static::lazy_static;
use libc::*;
use lockfree::map::Map;
use std::alloc::{alloc_zeroed, Layout};
use std::fmt::{Debug, Formatter};
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref SIGNAL_INSTALLED: AtomicBool = AtomicBool::new(false);
    static ref MEMORY_LIST: Map<usize, FastMemUnixPtr> = Map::new();
}

struct FastMemUnixPtr(*const FastMemUnix);
unsafe impl Send for FastMemUnixPtr {}
unsafe impl Sync for FastMemUnixPtr {}

const TOTAL_ALLOC_SIZE: size_t = usize::wrapping_add(0xffffffff, 4096);

pub struct FastMemUnix {
    base_addr: *mut u8,
    allocated: [AtomicBool; 1048576],
}

unsafe impl Send for FastMemUnix {}
unsafe impl Sync for FastMemUnix {}

impl Debug for FastMemUnix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastMemUnix")
            .field("base_addr", &self.base_addr)
            .finish()
    }
}

impl FastMemUnix {
    pub fn try_new(endian: EndianMode, segments: &[Segment]) -> Option<Box<Self>> {
        if endian != EndianMode::native() || size_of::<usize>() <= size_of::<u32>() {
            // non-native endian or 32-bit system
            return None;
        }

        let mut obj = unsafe {
            if sysconf(_SC_PAGESIZE) != 4096 {
                // non standard page size
                return None;
            }

            let base_addr = mmap(
                null_mut(),
                TOTAL_ALLOC_SIZE,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            ) as *mut u8;
            if base_addr.is_null() {
                return None;
            }

            let ptr = alloc_zeroed(Layout::new::<FastMemUnix>()) as *mut FastMemUnix;
            if ptr.is_null() {
                munmap(base_addr as *mut c_void, TOTAL_ALLOC_SIZE);
                return None;
            }

            // init fields
            (&mut (*ptr).base_addr as *mut *mut u8).write(base_addr);
            // finish init fields (`allocated` is skipped because zero is valid)

            let obj = Box::from_raw(ptr);
            // now it's boxed. no longer need to clean up pointers.

            // add to memory list
            MEMORY_LIST.insert(base_addr as usize, FastMemUnixPtr(ptr));

            // install signal handler
            if try_install_signal_handler().is_err() {
                return None;
            }

            obj
        };

        for seg in segments {
            obj.write_from_slice(seg.base_addr, &seg.data);
        }

        Some(obj)
    }

    // this function may be called in a signal handler
    unsafe fn map_page(&self, page_idx: usize) -> Result<(), c_int> {
        let prev = self.allocated[page_idx].swap(true, Ordering::SeqCst);
        if prev {
            return Ok(());
        }

        let host_addr = self.base_addr.add(page_idx * 4096);
        let status = mprotect(host_addr as *mut c_void, 4096, PROT_READ | PROT_WRITE);

        if status == 0 {
            Ok(())
        } else {
            Err(*__errno_location())
        }
    }
}

impl FastMem for FastMemUnix {
    fn fastmem_addr(&self) -> *mut u8 {
        self.base_addr
    }
}

impl Drop for FastMemUnix {
    fn drop(&mut self) {
        // remove signal handler
        if MEMORY_LIST.remove(&(self.base_addr as usize)).is_none() {
            eprintln!("unable to find self from MEMORY_LIST");
        }

        // unmap memory
        unsafe {
            munmap(self.base_addr as *mut c_void, TOTAL_ALLOC_SIZE);
        }

        // clean up myself
        self.base_addr = null_mut();
    }
}

unsafe fn try_install_signal_handler() -> Result<(), c_int> {
    let was_installed = SIGNAL_INSTALLED.swap(true, Ordering::SeqCst);
    if was_installed {
        return Ok(());
    }

    // touch it to allocate before signal handler accesses it
    MEMORY_LIST.iter();

    let mut mask = zeroed();
    sigemptyset(&mut mask);

    let act = sigaction {
        sa_sigaction: handler as usize,
        sa_mask: mask,
        sa_flags: SA_SIGINFO | SA_NODEFER | SA_RESTART,
        sa_restorer: None,
    };

    let status = sigaction(SIGSEGV, &act, null_mut());

    if status == 0 {
        Ok(())
    } else {
        Err(*__errno_location())
    }
}

unsafe extern "system" fn handler(sig_num: c_int, info: *const siginfo_t, _ctx: *mut c_void) {
    if sig_num != SIGSEGV {
        let msg = "SIGSEGV handler received wrong signal\n";
        write(STDERR_FILENO, msg.as_ptr() as *mut c_void, msg.len());
        abort();
    }

    let addr = (*info).si_addr() as usize;

    for x in MEMORY_LIST.iter() {
        let base_addr = *x.key();
        let mem = x.val().0;

        if addr < base_addr || base_addr + 0xffff_ffff < addr {
            // invalid address
            continue;
        }

        // let's map the area
        let page_idx = (addr - base_addr) / 4096;
        match (*mem).map_page(page_idx) {
            Ok(_) => return,
            Err(_) => {
                let msg = "SIGSEGV caught and failed to map page\n";
                write(STDERR_FILENO, msg.as_ptr() as *mut c_void, msg.len());
                abort();
            }
        }
    }

    // did not find appropriate area
    let msg = "SIGSEGV caught and did not find appropriate instance\n";
    write(STDERR_FILENO, msg.as_ptr() as *mut c_void, msg.len());
    abort();
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

        let mut mem = FastMemUnix::try_new(EndianMode::native(), &[]).expect("creating memory");

        mem.write_u32(0x12345678, 4321);
        assert_eq!(mem.read_u32(0x12345678), 4321);

        assert!(mem.allocated[0x12345678 / 4096].load(Ordering::SeqCst));
    }

    #[test]
    fn multiple_instance() {
        let _guard = TEST_LOCK.lock();

        let mut mem1 = FastMemUnix::try_new(EndianMode::native(), &[]).expect("creating memory");
        let mut mem2 = FastMemUnix::try_new(EndianMode::native(), &[]).expect("creating memory");

        mem1.write_u32(0x1234, 1);
        mem2.write_u32(0x4321, 2);
        assert_eq!(mem1.read_u32(0x1234), 1);
        assert_eq!(mem2.read_u32(0x4321), 2);

        // check if corresponding pages are allocated
        assert!(mem1.allocated[1].load(Ordering::SeqCst));
        assert!(mem2.allocated[4].load(Ordering::SeqCst));

        drop(mem1);
        drop(mem2);
    }
}
