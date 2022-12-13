mod emptymem;
mod endian_mode;
mod memory_trait;
mod segment;
mod slowmem;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod fastmem_windows;
    } else if #[cfg(unix)] {
        mod fastmem_unix;
    }
}

pub use endian_mode::EndianMode;
pub use memory_trait::{create_empty_memory, create_memory, create_memory_fastmem, Memory};
pub use segment::Segment;
