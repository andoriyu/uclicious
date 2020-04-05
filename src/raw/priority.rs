//! Source priority. Consult libUCL documentation for more information.
use std::os::raw::c_uint;
/// Priorities are used by UCL parser to manage the policy of objects rewriting during including other files as following:
/// - If we have two objects with the same priority then we form an implicit array
/// - If a new object has bigger priority then we overwrite an old one
/// - If a new object has lower priority then we ignore it
///
/// By default, the priority of top-level object is set to zero (the lowest priority). Currently, you can define up to 16 priorities (from 0 to 15).
/// Includes with bigger priorities will rewrite keys from the objects with lower priorities as specified by the policy.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Priority(c_uint);

impl Priority {
    #[inline(always)]
    fn normalize_unsigned(source: u32) -> Priority {
        let priority = if source > 15 { 15 } else { source };
        Priority(priority)
    }

    #[inline(always)]
    fn normalize_signed(source: i64) -> Priority {
        let priority = if source > 15 {
            15
        } else if source < 0 {
            0
        } else {
            source
        };
        Priority(priority as u32)
    }

    /// Create a Priority. Values outside of 0..15 range will be changed to nearest "legal" number.
    #[inline(always)]
    pub fn new(priority: u32) -> Priority {
        Priority::normalize_unsigned(priority)
    }

    #[inline(always)]
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
        Priority::normalize_unsigned(source as u32)
    }
}

impl From<u32> for Priority {
    fn from(priority: u32) -> Self {
        Priority::normalize_unsigned(priority)
    }
}
impl From<u16> for Priority {
    fn from(priority: u16) -> Self {
        Priority::normalize_unsigned(priority as u32)
    }
}
impl From<u8> for Priority {
    fn from(priority: u8) -> Self {
        Priority::normalize_unsigned(priority as u32)
    }
}
impl From<i64> for Priority {
    fn from(source: i64) -> Self {
        Priority::normalize_signed(source)
    }
}

impl From<i32> for Priority {
    fn from(priority: i32) -> Self {
        Priority::normalize_signed(priority as i64)
    }
}
impl From<i16> for Priority {
    fn from(priority: i16) -> Self {
        Priority::normalize_signed(priority as i64)
    }
}

impl From<i8> for Priority {
    fn from(priority: i8) -> Self {
        Priority::normalize_signed(priority as i64)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::From;
    use std::convert::Into;

    #[test]
    fn test_native_type() {
        let priority = Priority::new(1);
        assert_eq!(1, priority.as_c_uint());

        let higher = Priority::new(256);
        assert_eq!(15, higher.as_c_uint());

        let zero = Priority::new(0);
        assert_eq!(0, zero.as_c_uint());

        let default = Priority::default();
        assert_eq!(0, default.as_c_uint());
    }

    #[test]
    fn test_from_trait_unsigned_okay() {
        let expected = Priority::new(1);
        let from_u8: Priority = 1u8.into();
        assert_eq!(expected, from_u8);

        let from_u32: Priority = 1u32.into();
        assert_eq!(expected, from_u32);

        let from_u16: Priority = 1u16.into();
        assert_eq!(expected, from_u16);

        let from_u64: Priority = 1u64.into();
        assert_eq!(expected, from_u64);
    }

    #[test]
    fn test_from_trait_unsigned_higher() {
        let expected = Priority::new(15);
        let from_u8: Priority = 42u8.into();
        assert_eq!(expected, from_u8);

        let from_u16: Priority = 256u16.into();
        assert_eq!(expected, from_u16);

        let from_u32: Priority = 300u32.into();
        assert_eq!(expected, from_u32);

        let from_u64: Priority = 69420u64.into();
        assert_eq!(expected, from_u64);
    }

    #[test]
    fn test_from_trait_signed_okay() {
        let expected = Priority::new(1);
        let from_i8: Priority = 1i8.into();
        assert_eq!(expected, from_i8);

        let from_i32: Priority = 1i32.into();
        assert_eq!(expected, from_i32);

        let from_i16: Priority = 1i16.into();
        assert_eq!(expected, from_i16);

        let from_i64: Priority = 1i64.into();
        assert_eq!(expected, from_i64);
    }

    #[test]
    fn test_from_trait_signed_higher() {
        let expected = Priority::new(15);
        let from_i8: Priority = 42i8.into();
        assert_eq!(expected, from_i8);

        let from_i16: Priority = 256i16.into();
        assert_eq!(expected, from_i16);

        let from_i32: Priority = 300i32.into();
        assert_eq!(expected, from_i32);

        let from_i64: Priority = 69420i64.into();
        assert_eq!(expected, from_i64);
    }

    #[test]
    fn test_from_trait_signed_lower() {
        let expected = Priority::new(0);
        let from_i8: Priority = Priority::from(-4i8);
        assert_eq!(expected, from_i8);

        let from_i16: Priority = (-256i16).into();
        assert_eq!(expected, from_i16);

        let from_i32: Priority = (-300i32).into();
        assert_eq!(expected, from_i32);

        let from_i64: Priority = (-69420i64).into();
        assert_eq!(expected, from_i64);
    }
}
