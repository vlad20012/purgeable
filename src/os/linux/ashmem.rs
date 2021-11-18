use crate::PurgeableAllocError;
use std::alloc::Layout;
use std::mem::ManuallyDrop;
use std::ptr;
use std::ptr::NonNull;

mod ashmem_sys;

pub(crate) struct SystemPurgeableBox<T: ?Sized> {
    ptr: ptr::NonNull<T>,
    fd: libc::c_int,
    pub(crate) size: usize, // Remove it after `size_of_val_raw` stabilization
}

impl SystemPurgeableBox<[u8]> {
    pub(crate) fn new_uninit_with_layout(
        layout: Layout,
    ) -> Result<SystemPurgeableBox<[u8]>, PurgeableAllocError> {
        super::super::check_alignment(layout);
        if layout.size() == 0 {
            return Ok(SystemPurgeableBox {
                ptr: unsafe {
                    NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                        layout.align() as *mut u8,
                        0,
                    ))
                },
                fd: 0,
                size: 0,
            });
        }

        let fd = unsafe { ashmem_sys::create(ptr::null(), layout.size() as libc::size_t) };
        if fd < 0 {
            // return Err(io::Error::last_os_error());
            return Err(PurgeableAllocError::new(layout));
        }

        let address = unsafe {
            libc::mmap(
                ptr::null_mut(),
                layout.size() as libc::size_t,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if address == libc::MAP_FAILED {
            // return Err(io::Error::last_os_error());
            return Err(PurgeableAllocError::new(layout));
        }

        let ptr = unsafe {
            ptr::NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                address as *mut u8,
                layout.size(),
            ))
        };

        Ok(SystemPurgeableBox {
            ptr,
            fd,
            size: layout.size(),
        })
    }
}

impl<T: ?Sized> Drop for SystemPurgeableBox<T> {
    fn drop(&mut self) {
        if self.fd == 0 {
            return;
        }
        unsafe {
            libc::munmap(self.ptr() as *mut _, self.size());
            libc::close(self.fd);
        }
    }
}

impl<T: ?Sized> SystemPurgeableBox<T> {
    pub(crate) fn lock(&self) -> bool {
        if self.fd == 0 {
            return true;
        }
        unsafe { ashmem_sys::pin(self.fd) }
    }

    pub(crate) unsafe fn unlock(&self) {
        if self.fd == 0 {
            return;
        }
        ashmem_sys::unpin(self.fd);
    }

    #[inline]
    pub(crate) fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub(crate) unsafe fn map_ptr<R: ?Sized>(
        self,
        f: impl FnOnce(ptr::NonNull<T>) -> ptr::NonNull<R>,
    ) -> SystemPurgeableBox<R> {
        let s = ManuallyDrop::new(self);
        let ptr = s.ptr;
        let fd = s.fd;
        let size = s.size;
        return SystemPurgeableBox {
            ptr: f(ptr),
            fd,
            size,
        };
    }
}

pub fn is_available() -> bool {
    SystemPurgeableBox::<[u8]>::new_uninit_with_layout(Layout::new::<u8>()).is_ok()
}
