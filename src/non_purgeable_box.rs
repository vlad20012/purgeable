use crate::error::PurgeableBoxLockError;
use crate::unsafe_purgeable_box::UnsafePurgeableBox;
use crate::{PurgeableAllocError, PurgeableBox};
use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::{fmt, ops};

/// # Examples
///
/// ```
/// use purgeable::{NonPurgeableBox};
///
/// let npb = NonPurgeableBox::new(&1);
/// let pb = NonPurgeableBox::unlock(npb);
///
/// match pb.lock() {
///     Ok(boxed) => assert_eq!(*boxed, 1),
///     Err(_) => println!("The purgeable box has been purged")
/// }
/// ```
pub struct NonPurgeableBox<T: ?Sized> {
    // Invariant: `inner` is in the `LOCKED` state
    inner: UnsafePurgeableBox<T>,
}

impl<T: Copy> NonPurgeableBox<T> {
    pub fn new(x: &T) -> NonPurgeableBox<T> {
        handle_alloc_result(Self::try_new(x))
    }

    pub fn new_uninit() -> NonPurgeableBox<MaybeUninit<T>> {
        handle_alloc_result(Self::try_new_uninit())
    }

    pub fn try_new(x: &T) -> Result<NonPurgeableBox<T>, PurgeableAllocError> {
        let mut npb = Self::try_new_uninit()?;

        // SAFETY: `npb.as_mut_ptr()` points to an allocated memory for size of `T`,
        //  `x` points to a `T` reference, `T` is `Copy`
        let bx = unsafe {
            npb.as_mut_ptr().copy_from_nonoverlapping(x, 1);
            npb.assume_init()
        };

        Ok(bx)
    }

    pub fn try_new_uninit() -> Result<NonPurgeableBox<MaybeUninit<T>>, PurgeableAllocError> {
        let locked_inner = UnsafePurgeableBox::try_new_locked_uninit()?;
        // SAFETY: `try_new_locked_uninit` guarantees that `locked_inner` is in the `LOCKED` state
        let npb = unsafe { NonPurgeableBox::from_locked_inner(locked_inner) };
        Ok(npb)
    }
}

impl<T: Copy> NonPurgeableBox<[T]> {
    pub fn new_filled_slice(x: T, len: usize) -> NonPurgeableBox<[T]> {
        let mut npb = Self::new_uninit_slice(len);
        npb.fill(MaybeUninit::new(x));
        // SAFETY: `npb` is fully init because we just filled it with initialized values
        unsafe { npb.assume_init() }
    }

    pub fn new_uninit_slice(len: usize) -> NonPurgeableBox<[MaybeUninit<T>]> {
        handle_alloc_result(Self::try_new_uninit_slice(len))
    }

    pub fn try_new_uninit_slice(
        len: usize,
    ) -> Result<NonPurgeableBox<[MaybeUninit<T>]>, PurgeableAllocError> {
        let locked_inner = UnsafePurgeableBox::<[T]>::try_new_locked_uninit_slice(len)?;
        // SAFETY:
        // `try_new_locked_uninit_slice` guarantees that `locked_inner` is in the `LOCKED` state
        let npb = unsafe { NonPurgeableBox::from_locked_inner(locked_inner) };
        Ok(npb)
    }

    pub fn new_slice(src: &[T]) -> NonPurgeableBox<[T]> {
        handle_alloc_result(Self::try_new_slice(src))
    }

    pub fn try_new_slice(src: &[T]) -> Result<NonPurgeableBox<[T]>, PurgeableAllocError> {
        let mut pb = Self::try_new_uninit_slice(src.len())?;

        // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
        let uninit_src: &[MaybeUninit<T>] = unsafe { std::mem::transmute(src) };

        pb.copy_from_slice(uninit_src);

        // SAFETY: `pb` is now fully initialized because it has the same length as `uninit_src`
        let pb = unsafe { pb.assume_init() };
        Ok(pb)
    }
}

impl<T: ?Sized> NonPurgeableBox<T> {
    /// Safety: `pb` must be in the `LOCKED` state
    unsafe fn from_locked_inner(inner: UnsafePurgeableBox<T>) -> Self {
        NonPurgeableBox { inner }
    }

    /// Safety: `pb` must be in the `UNLOCKED` state
    pub(crate) unsafe fn try_from_unlocked(
        mut pb: UnsafePurgeableBox<T>,
    ) -> Result<NonPurgeableBox<T>, PurgeableBoxLockError> {
        // SAFETY: the caller must guarantee that `pb` is in the `UNLOCKED` state
        if pb.lock() {
            // SAFETY: `pb.lock()` returned true, so `pb` is now in the `LOCKED` state
            Ok(Self::from_locked_inner(pb))
        } else {
            // drop(self);
            Err(PurgeableBoxLockError)
        }
    }

    pub fn unlock(this: Self) -> PurgeableBox<T> {
        let mut pb = this.inner;
        // SAFETY: `NonPurgeableBox` guarantees that `pb` is in the `LOCKED` state;
        //  then we're turning it into `UNLOCKED` state by calling `unlock`;
        //  then we're passing it to `PurgeableBox::from_unlocked` which requires `UNLOCKED` state.
        unsafe {
            pb.unlock();
            PurgeableBox::from_unlocked(pb)
        }
    }
}

impl<T: Copy> NonPurgeableBox<[MaybeUninit<T>]> {
    /// See docs for [MaybeUninit::assume_init]
    #[inline(always)]
    pub unsafe fn assume_init(self) -> NonPurgeableBox<[T]> {
        // SAFETY: `NonPurgeableBox` guarantees that `self.inner` is in the `LOCKED` state
        NonPurgeableBox::from_locked_inner(self.inner.assume_init())
    }
}

impl<T> NonPurgeableBox<MaybeUninit<T>> {
    /// See docs for [MaybeUninit::assume_init]
    #[inline(always)]
    unsafe fn assume_init(self) -> NonPurgeableBox<T> {
        NonPurgeableBox {
            inner: self.inner.assume_init(),
        }
    }
}

/// `NonPurgeableBox` pointers are `Sync` if `T` is `Sync` because the data they
/// reference is unaliased.
unsafe impl<T: Sync + ?Sized> Sync for NonPurgeableBox<T> {}

impl<T: fmt::Display + ?Sized> fmt::Display for NonPurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for NonPurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Pointer for NonPurgeableBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}

impl<T: Copy> Clone for NonPurgeableBox<T> {
    fn clone(&self) -> Self {
        NonPurgeableBox::<T>::new(self)
    }
}

impl<T: ?Sized> ops::Deref for NonPurgeableBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `NonPurgeableBox` guarantees that `self.inner` is in the `LOCKED` state
        unsafe { self.inner.as_ref() }
    }
}

impl<T: ?Sized> ops::DerefMut for NonPurgeableBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: `NonPurgeableBox` guarantees that `self.inner` is in the `LOCKED` state
        unsafe { self.inner.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for NonPurgeableBox<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T: ?Sized> AsMut<T> for NonPurgeableBox<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut *self
    }
}

impl<T: ?Sized> Borrow<T> for NonPurgeableBox<T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<T: ?Sized> BorrowMut<T> for NonPurgeableBox<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut *self
    }
}

impl<T: ?Sized + PartialEq> PartialEq for NonPurgeableBox<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for NonPurgeableBox<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for NonPurgeableBox<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for NonPurgeableBox<T> {}

impl<T: ?Sized + Hash> Hash for NonPurgeableBox<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

fn handle_alloc_result<T: ?Sized>(
    result: Result<NonPurgeableBox<T>, PurgeableAllocError>,
) -> NonPurgeableBox<T> {
    match result {
        Ok(ok) => ok,
        Err(e) => {
            panic!(
                "NonPurgeableBox memory allocation of {} bytes failed",
                e.layout.size()
            )
        }
    }
}

impl<T: ?Sized> TryFrom<PurgeableBox<T>> for NonPurgeableBox<T> {
    type Error = PurgeableBoxLockError;

    fn try_from(pb: PurgeableBox<T>) -> Result<Self, Self::Error> {
        pb.lock()
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use crate::NonPurgeableBox;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<T: Serialize + ?Sized> Serialize for NonPurgeableBox<T> {
        #[inline]
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            (**self).serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de> + Copy> Deserialize<'de> for NonPurgeableBox<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer).map(|x| NonPurgeableBox::new(&x))
        }
    }

    impl<'de, T: Deserialize<'de> + Copy> Deserialize<'de> for NonPurgeableBox<[T]> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer).map(|v: Vec<T>| Self::new_slice(&v))
        }
    }
}

#[cfg(feature = "stable_deref_trait")]
mod stable_deref_trait_impls {
    use crate::NonPurgeableBox;

    unsafe impl<T: ?Sized> stable_deref_trait::StableDeref for NonPurgeableBox<T> {}
}
