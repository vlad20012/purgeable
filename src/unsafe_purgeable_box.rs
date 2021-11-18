use crate::error::PurgeableAllocError;
use crate::os;
use std::fmt;
use std::mem::MaybeUninit;

/// States: `LOCKED`, `UNLOCKED`, `PURGED`.
pub(crate) struct UnsafePurgeableBox<T: ?Sized> {
    inner: os::SystemPurgeableBox<T>,
}

impl<T: Copy> UnsafePurgeableBox<T> {
    /// Returns the box in the `LOCKED` state
    pub(crate) fn try_new_locked_uninit(
    ) -> Result<UnsafePurgeableBox<MaybeUninit<T>>, PurgeableAllocError> {
        let inner = os::SystemPurgeableBox::new_uninit()?;
        Ok(UnsafePurgeableBox { inner })
    }
}

impl<T: ?Sized> UnsafePurgeableBox<T> {
    /// # Safety
    ///
    /// The caller must guarantee that `self` is in the `UNLOCKED` state.
    /// It is *[UB]* to call `lock` on an [UnsafePurgeableBox] that is in a different state.
    /// Note that `lock` changes the state regardless of the `lock` return value, hence
    /// calling `lock` twice is always an *[UB]*.
    ///
    /// [UB]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[must_use]
    pub(crate) unsafe fn lock(&mut self) -> bool {
        self.inner.lock()
    }

    /// # Safety
    ///
    /// The caller must guarantee that `self` is in the `LOCKED` state.
    pub(crate) unsafe fn unlock(&mut self) {
        self.inner.unlock()
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub(crate) fn is_purged(&self) -> bool {
        self.inner.is_purged()
    }

    /// # Safety
    ///
    /// Calling `ptr` is always safe, but accessing a content behind the pointer is safe only
    /// if `self` is in the `LOCKED` state.
    #[inline]
    fn ptr(&self) -> *mut T {
        self.inner.ptr()
    }

    /// # Safety
    ///
    /// The caller must guarantee that `self` is in the `LOCKED` state.
    #[inline]
    pub(crate) unsafe fn as_ref(&self) -> &T {
        &*self.ptr()
    }

    /// # Safety
    ///
    /// The caller must guarantee that `self` is in the `LOCKED` state.
    #[inline]
    pub(crate) unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.ptr()
    }

    pub(crate) fn size(&self) -> usize {
        self.inner.size()
    }
}

impl<T: Copy> UnsafePurgeableBox<[T]> {
    pub(crate) fn try_new_locked_uninit_slice(
        len: usize,
    ) -> Result<UnsafePurgeableBox<[MaybeUninit<T>]>, PurgeableAllocError> {
        let inner = os::SystemPurgeableBox::<[T]>::new_uninit_slice(len)?;
        Ok(UnsafePurgeableBox { inner })
    }
}

impl<T: Copy> UnsafePurgeableBox<[MaybeUninit<T>]> {
    /// See docs for [MaybeUninit::assume_init]
    #[inline(always)]
    pub(crate) unsafe fn assume_init(self) -> UnsafePurgeableBox<[T]> {
        UnsafePurgeableBox {
            inner: self.inner.assume_init(),
        }
    }
}

impl<T> UnsafePurgeableBox<MaybeUninit<T>> {
    /// See docs for [MaybeUninit::assume_init]
    #[inline(always)]
    pub(crate) unsafe fn assume_init(self) -> UnsafePurgeableBox<T> {
        UnsafePurgeableBox {
            inner: self.inner.assume_init(),
        }
    }
}

impl<T: ?Sized> fmt::Pointer for UnsafePurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr(), f)
    }
}
