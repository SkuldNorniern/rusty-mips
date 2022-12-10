mod endian_mode;
mod memory_trait;
mod segment;
mod slowmem;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod fastmem_windows;
    }
}

pub use endian_mode::EndianMode;
pub use memory_trait::{create_memory, Memory};
pub use segment::Segment;
