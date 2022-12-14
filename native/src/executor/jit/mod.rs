cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x64;
        pub use x64::X64Jit as Jit;
        pub const HAS_JIT: bool = true;
    } else {
        mod dummy;
        pub use dummy::DummyJit as Jit;
        pub const HAS_JIT: bool = false;
    }
}
