#[cfg(any(target_os = "macos", target_os = "ios"))]
mod mach;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) use mach::{is_available, SystemPurgeableBox};
use std::alloc::Layout;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::{is_available, SystemPurgeableBox};

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) use linux::{is_available, SystemPurgeableBox};

mod impls;

fn check_alignment(layout: Layout) {
    if layout.align() > page_size::get() {
        panic!(
            "Requested alignment is larger than page size: {} > {}; this is not supported yet",
            layout.align(),
            page_size::get()
        );
    }
}
