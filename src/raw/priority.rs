//! Source priority. Consult libUCL documentation for more information.
use std::os::raw::c_uint;
/// Priorities are used by UCL parser to manage the policy of objects rewriting during including other files as following:
/// - If we have two objects with the same priority then we form an implicit array
/// - If a new object has bigger priority then we overwrite an old one
/// - If a new object has lower priority then we ignore it
///
/// By default, the priority of top-level object is set to zero (the lowest priority). Currently, you can define up to 16 priorities (from 0 to 15).
/// Includes with bigger priorities will rewrite keys from the objects with lower priorities as specified by the policy.
pub struct Priority(c_uint);

impl Priority {
    pub fn new(priority: u32) -> Priority {
        Priority(priority)
    }

    pub fn as_c_uint(&self) -> c_uint {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority(0)
    }
}

impl From<u64> for Priority {
    fn from(source: u64) -> Self {
        let priority = {
            if source > 15 {
                15
            } else {
                source
            }
        };
        Priority(priority as u32)
    }
}

impl From<u32> for Priority {
    fn from(priority: u32) -> Self {
        Self::from(priority as u64)
    }
}
impl From<u16> for Priority {
    fn from(priority: u16) -> Self {
        Self::from(priority as u64)
    }
}
impl From<i64> for Priority {
    fn from(source: i64) -> Self {
        let priority = {
            if source > 15 {
                15
            } else if source < 0 {
                0
            } else {
                source
            }
        };
        Priority(priority as u32)
    }
}

impl From<i32> for Priority {
    fn from(priority: i32) -> Self {
        Self::from(priority as i64)
    }
}
impl From<i16> for Priority {
    fn from(priority: i16) -> Self {
        Self::from(priority as i64)
    }
}
