use crate::PurgeableAllocError;
use mach_sys::{
    vm_address_t, vm_size_t, KERN_SUCCESS, VM_FLAGS_ANYWHERE, VM_FLAGS_PURGABLE, VM_PURGABLE_EMPTY,
    VM_PURGABLE_GET_STATE, VM_PURGABLE_NONVOLATILE, VM_PURGABLE_SET_STATE, VM_PURGABLE_VOLATILE,
    VM_VOLATILE_GROUP_DEFAULT,
};
use std::alloc::Layout;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::{ptr};

mod mach_sys;

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

        let mut address: vm_address_t = 0;
        let result = unsafe {
            mach_sys::vm_allocate(
                mach_sys::mach_task_self(),
                &mut address as *mut _,
                layout.size() as vm_size_t,
                VM_FLAGS_PURGABLE | VM_FLAGS_ANYWHERE,
            )
        };

        if result != KERN_SUCCESS || address == 0 {
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
            mach_sys::vm_deallocate(
                mach_sys::mach_task_self(),
                self.ptr.as_ptr() as *mut c_void as vm_address_t,
                self.size() as vm_size_t,
            );
        }
    }
}

impl<T: ?Sized> SystemPurgeableBox<T> {
    pub(crate) fn lock(&self) -> bool {
        if self.size == 0 {
            return true;
        }

        let mut state = VM_PURGABLE_NONVOLATILE;

        let ret = unsafe {
            mach_sys::vm_purgable_control(
                mach_sys::mach_task_self(),
                self.ptr.as_ptr() as *mut c_void as vm_address_t,
                VM_PURGABLE_SET_STATE,
                &mut state,
            )
        };

        if ret != KERN_SUCCESS {
            return false;
        }

        return state & VM_PURGABLE_EMPTY == 0;
    }

    pub(crate) unsafe fn unlock(&self) {
        if self.size == 0 {
            return;
        }

        let mut state = VM_PURGABLE_VOLATILE | VM_VOLATILE_GROUP_DEFAULT;

        let ret = mach_sys::vm_purgable_control(
            mach_sys::mach_task_self(),
            self.ptr.as_ptr() as *mut c_void as vm_address_t,
            VM_PURGABLE_SET_STATE,
            &mut state,
        );

        debug_assert_eq!(ret, KERN_SUCCESS)
    }

    pub(crate) fn is_purged(&self) -> bool {
        let mut state = 0;

        let ret = unsafe {
            mach_sys::vm_purgable_control(
                mach_sys::mach_task_self(),
                self.ptr.as_ptr() as *mut c_void as vm_address_t,
                VM_PURGABLE_GET_STATE,
                &mut state,
            )
        };

        if ret != KERN_SUCCESS {
            return true;
        }

        return state & VM_PURGABLE_EMPTY != 0;
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
