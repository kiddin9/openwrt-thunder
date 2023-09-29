#[cfg(all(target_os = "linux", target_env = "musl"))]
pub mod libc;
pub mod xunlei;
