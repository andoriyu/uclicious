//! Clone of TryFrom from std. A hack around std's blanket implementations.
use crate::{ObjectError};

/// Implement this trait on your types in order for automatic derive to work.
pub trait FromObject<T>: Sized {
    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, ObjectError>;
}
