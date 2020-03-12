use crate::{ObjectError};

pub trait FromObject<T>: Sized {
    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, ObjectError>;
}
