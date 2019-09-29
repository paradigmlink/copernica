#![allow(non_camel_case_types, non_snake_case, dead_code)]

#[cfg(unix)]
pub use libc::*;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[path = "./linux.rs"]
mod platform;


#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[path = "./bpf.rs"]
mod bpf;

#[cfg(target_os = "macos")]
#[path = "./macos.rs"]
mod platform;


#[cfg(any(
    target_os = "macos",
    target_os = "android", target_os = "linux",
))]
pub use self::platform::*;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub use self::bpf::*;