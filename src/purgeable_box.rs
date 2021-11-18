use crate::error::PurgeableBoxLockError;
use crate::non_purgeable_box::NonPurgeableBox;
use crate::unsafe_purgeable_box::UnsafePurgeableBox;
use std::fmt;

pub struct PurgeableBox<T: ?Sized> {
    // Invariant: `inner` is in the `UNLOCKED` state
    inner: UnsafePurgeableBox<T>,
}

impl<T: ?Sized> PurgeableBox<T> {
    /// Safety: `pb` must be in the `UNLOCKED` state
    pub(crate) unsafe fn from_unlocked(pb: UnsafePurgeableBox<T>) -> PurgeableBox<T> {
        // SAFETY: the caller must guarantee that `pb` is in the `UNLOCKED` state
        PurgeableBox { inner: pb }
    }

    pub fn lock(self) -> Result<NonPurgeableBox<T>, PurgeableBoxLockError> {
        // SAFETY: `PurgeableBox` guarantees that `self.inner` is in the `UNLOCKED` state
        unsafe { NonPurgeableBox::try_from_unlocked(self.inner) }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn is_purged(&self) -> bool {
        self.inner.is_purged()
    }

    pub fn size(&self) -> usize {
        self.inner.size()
    }
}

impl<T: ?Sized> fmt::Pointer for PurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}

impl<T: ?Sized> fmt::Debug for PurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PurgeableBox")
    }
}

impl<T: ?Sized> From<NonPurgeableBox<T>> for PurgeableBox<T> {
    fn from(npb: NonPurgeableBox<T>) -> Self {
        NonPurgeableBox::unlock(npb)
    }
}
