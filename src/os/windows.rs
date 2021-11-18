use crate::PurgeableAllocError;
use std::alloc::Layout;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr;
use std::ptr::NonNull;
use winapi::shared::basetsd::SIZE_T;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, MEM_RESET, MEM_RESET_UNDO, PAGE_READWRITE,
};

pub(crate) struct SystemPurgeableBox<T: ?Sized> {
    ptr: NonNull<T>,
    pub(crate) size: usize, // Remove it after `size_of_val_raw` stabilization
}

impl SystemPurgeableBox<[u8]> {
    pub(crate) fn new_uninit_with_layout(
        layout: Layout,
    ) -> Result<SystemPurgeableBox<[u8]>, PurgeableAllocError> {
        super::check_alignment(layout);
        if layout.size() == 0 {
            return Ok(SystemPurgeableBox {
                ptr: unsafe {
                    NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                        layout.align() as *mut u8,
                        0,
                    ))
                },
                size: 0,
            });
        }

        let address = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                layout.size() as SIZE_T,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };

        if address.is_null() {
            return Err(PurgeableAllocError::new(layout));
        }

        let ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                address as *mut u8,
                layout.size(),
            ))
        };

        Ok(SystemPurgeableBox {
            ptr,
            size: layout.size(),
        })
    }
}

impl<T: ?Sized> Drop for SystemPurgeableBox<T> {
    fn drop(&mut self) {
        if self.size == 0 {
            return;
        }
        unsafe {
            VirtualFree(self.ptr.as_ptr() as *mut c_void, 0, MEM_RELEASE);
        }
    }
}

impl<T: ?Sized> SystemPurgeableBox<T> {
    pub(crate) fn lock(&self) -> bool {
        if self.size == 0 {
            return true;
        }

        let ret = unsafe {
            VirtualAlloc(
                self.ptr.as_ptr() as *mut c_void,
                self.size(),
                MEM_RESET_UNDO,
                PAGE_READWRITE,
            )
        };

        return !ret.is_null();
    }

    pub(crate) unsafe fn unlock(&self) {
        if self.size == 0 {
            return;
        }
        let ret = VirtualAlloc(
            self.ptr.as_ptr() as *mut c_void,
            self.size(),
            MEM_RESET,
            PAGE_READWRITE,
        );

        debug_assert!(!ret.is_null())
    }

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
        let size = s.size;
        return SystemPurgeableBox { ptr: f(ptr), size };
    }
}

pub fn is_available() -> bool {
    true
}
