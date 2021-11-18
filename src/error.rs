use std::alloc::Layout;
use std::error::Error;
use std::fmt;

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PurgeableAllocError {
    pub(crate) layout: Layout,
}

impl PurgeableAllocError {
    pub(crate) fn new(layout: Layout) -> PurgeableAllocError {
        PurgeableAllocError { layout }
    }
}

impl fmt::Display for PurgeableAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("purgeable memory allocation failed")
    }
}

impl Error for PurgeableAllocError {}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PurgeableBoxLockError;

impl fmt::Display for PurgeableBoxLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the purgeable box has already been purged")
    }
}

impl Error for PurgeableBoxLockError {}
