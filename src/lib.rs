mod os;

mod error;
mod non_purgeable_box;
mod purgeable_box;
mod unsafe_purgeable_box;

pub use non_purgeable_box::NonPurgeableBox;
pub use purgeable_box::PurgeableBox;

pub use error::{PurgeableAllocError, PurgeableBoxLockError};

pub fn is_available() -> bool {
    os::is_available()
}

#[cfg(test)]
mod tests;
