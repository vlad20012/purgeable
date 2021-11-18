use crate::os::SystemPurgeableBox;
use crate::PurgeableAllocError;
use std::alloc::Layout;
use std::mem::MaybeUninit;
use std::{mem, ptr};

impl<T: Copy> SystemPurgeableBox<T> {
    pub(crate) fn new_uninit() -> Result<SystemPurgeableBox<MaybeUninit<T>>, PurgeableAllocError> {
        SystemPurgeableBox::<[u8]>::new_uninit_with_layout(Layout::new::<T>())
            .map(|b| unsafe { b.cast() })
    }
}

impl<T: ?Sized> SystemPurgeableBox<T> {
    #[inline]
    pub(crate) unsafe fn cast<R>(self) -> SystemPurgeableBox<R> {
        return self.map_ptr(|ptr| ptr.cast());
    }

    pub(crate) fn size(&self) -> usize {
        // TODO replace to `size_of_val_raw` when it is stabilized
        self.size
    }
}

impl<T> SystemPurgeableBox<MaybeUninit<T>> {
    #[inline]
    pub(crate) unsafe fn assume_init(self) -> SystemPurgeableBox<T> {
        self.cast()
    }
}

impl<T: Copy> SystemPurgeableBox<[mem::MaybeUninit<T>]> {
    #[inline]
    pub(crate) unsafe fn assume_init(self) -> SystemPurgeableBox<[T]> {
        return self.map_ptr(|ptr| ptr::NonNull::new_unchecked(ptr.as_ptr() as *mut [T]));
    }
}

impl<T: Copy> SystemPurgeableBox<[T]> {
    pub(crate) fn new_uninit_slice(
        len: usize,
    ) -> Result<SystemPurgeableBox<[mem::MaybeUninit<T>]>, PurgeableAllocError> {
        let layout = Layout::array::<T>(len).unwrap();
        SystemPurgeableBox::<[u8]>::new_uninit_with_layout(layout).map(|b| unsafe {
            b.map_ptr(|ptr| {
                ptr::NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), len))
            })
        })
    }
}

/// `SystemPurgeableBox` pointers are `Send` if `T` is `Send` because the data they
/// reference is unaliased.
unsafe impl<T: Send + ?Sized> Send for SystemPurgeableBox<T> {}
