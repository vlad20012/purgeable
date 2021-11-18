// This file uses sources from https://github.com/kinetiknz/ashmem-rs distributed under
// ISC license (compatible with MIT and APACHE 2.0)

use ioctl_sys::iow;
use std::mem::size_of;

const __ASHMEMIOC: u32 = 0x77;

static mut LIBANDROID_ASHAREDMEMORY_CREATE: Option<
    extern "C" fn(*const libc::c_char, libc::size_t) -> libc::c_int,
> = None;
static mut LIBANDROID_ASHAREDMEMORY_GETSIZE: Option<extern "C" fn(libc::c_int) -> libc::size_t> =
    None;
static mut LIBANDROID_ASHAREDMEMORY_SETPROT: Option<
    extern "C" fn(libc::c_int, libc::c_int) -> libc::c_int,
> = None;

unsafe fn maybe_init() {
    const LIBANDROID_NAME: *const libc::c_char = "libandroid\0".as_ptr() as *const libc::c_char;
    const LIBANDROID_ASHAREDMEMORY_CREATE_NAME: *const libc::c_char =
        "ASharedMemory_create\0".as_ptr() as _;
    const LIBANDROID_ASHAREDMEMORY_GETSIZE_NAME: *const libc::c_char =
        "ASharedMemory_getSize\0".as_ptr() as _;
    const LIBANDROID_ASHAREDMEMORY_SETPROT_NAME: *const libc::c_char =
        "ASharedMemory_setProt\0".as_ptr() as _;
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // Leak the handle, there's no safe time to close it.
        let handle = libc::dlopen(LIBANDROID_NAME, libc::RTLD_LAZY | libc::RTLD_LOCAL);
        if handle.is_null() {
            return;
        }
        // Transmute guarantee for `fn -> Option<fn>`: https://doc.rust-lang.org/std/option/#representation
        LIBANDROID_ASHAREDMEMORY_CREATE =
            std::mem::transmute(libc::dlsym(handle, LIBANDROID_ASHAREDMEMORY_CREATE_NAME));
        LIBANDROID_ASHAREDMEMORY_GETSIZE =
            std::mem::transmute(libc::dlsym(handle, LIBANDROID_ASHAREDMEMORY_GETSIZE_NAME));
        LIBANDROID_ASHAREDMEMORY_SETPROT =
            std::mem::transmute(libc::dlsym(handle, LIBANDROID_ASHAREDMEMORY_SETPROT_NAME));
    });
}

/// See [ASharedMemory_create NDK documentation](https://developer.android.com/ndk/reference/group/memory#asharedmemory_create)
///
/// # Safety
///
/// Directly calls C or kernel APIs.
#[allow(non_snake_case)]
pub(crate) unsafe fn create(name: *const libc::c_char, size: libc::size_t) -> libc::c_int {
    const ASHMEM_NAME_DEF: *const libc::c_char = "/dev/ashmem\0".as_ptr() as _;
    const ASHMEM_NAME_LEN: usize = 256;
    const ASHMEM_SET_NAME: u32 = iow!(
        __ASHMEMIOC,
        1,
        std::mem::size_of::<[libc::c_char; ASHMEM_NAME_LEN]>()
    );
    const ASHMEM_SET_SIZE: u32 = iow!(__ASHMEMIOC, 3, std::mem::size_of::<libc::size_t>());

    maybe_init();
    if let Some(fun) = LIBANDROID_ASHAREDMEMORY_CREATE {
        return fun(name, size);
    }

    let fd = libc::open(ASHMEM_NAME_DEF, libc::O_RDWR, 0o600);
    if fd < 0 {
        return fd;
    }

    if !name.is_null() {
        // NOTE: libcutils uses a local stack copy of `name`.
        let r = libc::ioctl(fd, ASHMEM_SET_NAME as _, name);
        if r != 0 {
            libc::close(fd);
            return -1;
        }
    }

    let r = libc::ioctl(fd, ASHMEM_SET_SIZE as _, size);
    if r != 0 {
        libc::close(fd);
        return -1;
    }

    fd
}

#[repr(C)]
struct AshmemPin {
    offset: libc::__u32,
    len: libc::__u32,
}

const ASHMEM_NOT_PURGED: libc::c_int = 0;
// const ASHMEM_WAS_PURGED: libc::c_int = 1;
// const ASHMEM_IS_UNPINNED: libc::c_int = 0;
// const ASHMEM_IS_PINNED: libc::c_int = 1;

pub(crate) unsafe fn pin(fd: libc::c_int) -> bool {
    const ASHMEM_PIN: u32 = iow!(__ASHMEMIOC, 7, size_of::<AshmemPin>());
    let pin = AshmemPin { offset: 0, len: 0 };
    return libc::ioctl(fd, ASHMEM_PIN as _, &pin) == ASHMEM_NOT_PURGED;
}

pub(crate) unsafe fn unpin(fd: libc::c_int) {
    const ASHMEM_UNPIN: u32 = iow!(__ASHMEMIOC, 8, size_of::<AshmemPin>());
    let pin = AshmemPin { offset: 0, len: 0 };
    let _ = libc::ioctl(fd, ASHMEM_UNPIN as _, &pin);
}
