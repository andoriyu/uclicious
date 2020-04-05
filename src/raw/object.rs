//! Objects parsed by the parser.
//!
//! When you are done feeding the parser call `::get_object()` method on a parser. This will give you
//! an owned copy of an `Object`. Difference between `Object` and `ObjectRef` - internal reference count is only decreased for `Object` when dropped.
//!
//! ### Cloning
//!
//! Due to how rust std lib blanket impls work it's impossible to clone `ObjectRef` without converting it into owned `Object`.
//! `Object` implements clone by increasing reference count of object.
//!
//! #### Deep Cloning
//!
//! It's possible to create a deep copy of an `Object` and `ObjectRef` by calling `ObjectRef::deep_copy()`. Copy returned by that method is a completly different object with different address in memory.
//!
//! ### Equality and Ordering
//!
//! Literally all objects can be compared. The order:
//!
//! 1. Type of objects
//! 2. Size of objects
//! 3. Content of objects
//!
//! That means you can compare a string to float, and it will give some result. I'm not sure about usefulness of this, but it is totally possible.
use crate::raw::iterator::Iter;
use crate::raw::{utils, Priority};
use crate::traits::FromObject;
use bitflags::_core::borrow::Borrow;
use bitflags::_core::convert::Infallible;
use bitflags::_core::fmt::{Display, Formatter};
use libucl_bind::{
    ucl_object_frombool, ucl_object_fromdouble, ucl_object_fromint, ucl_object_fromstring,
    ucl_object_get_priority, ucl_object_key, ucl_object_lookup, ucl_object_lookup_path,
    ucl_object_ref, ucl_object_t, ucl_object_toboolean_safe, ucl_object_todouble_safe,
    ucl_object_toint_safe, ucl_object_tostring_forced, ucl_object_tostring_safe, ucl_object_type,
    ucl_object_unref, ucl_type_t, ucl_object_compare, ucl_object_copy
};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::hash::BuildHasher;
use std::mem::MaybeUninit;
use std::net::{AddrParseError, SocketAddr};
use std::num::TryFromIntError;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::Duration;
use bitflags::_core::cmp::Ordering;

/// Errors that could be returned by `Object` or `ObjectRef` functions.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ObjectError {
    KeyNotFound(String),
    /// Object was found, but value type doesn't match the desired type.
    ///
    /// NOTE: Error only returned when conversion is done by `FromObject` trait. Built-in functions return `None`.
    WrongType {
        key: String,
        actual_type: ucl_type_t,
        wanted_type: ucl_type_t,
    },
    /// Wrapper around `TryFromIntError`.
    IntConversionError(TryFromIntError),
    /// Wrapper around `AddrParseError`.
    AddrParseError(AddrParseError),
    /// An error that we couldn't match to internal type.
    Other(String),
    /// Not an error, but required for some conversions.
    None,
}

impl Error for ObjectError {}

impl ObjectError {
    /// Wrap error in Box<>.
    pub fn boxed(self) -> Box<ObjectError> {
        Box::new(self)
    }

    /// Wrap error in Box<> and erase its type.
    pub fn boxed_dyn(self) -> Box<dyn Error> {
        Box::new(self)
    }

    /// Create a new error `Other` by extracting the error description.
    pub fn other<E: Display>(err: E) -> ObjectError {
        ObjectError::Other(err.to_string())
    }
}
impl From<Infallible> for ObjectError {
    fn from(_: Infallible) -> Self {
        ObjectError::None
    }
}

impl From<AddrParseError> for ObjectError {
    fn from(source: AddrParseError) -> Self {
        ObjectError::AddrParseError(source)
    }
}
impl From<TryFromIntError> for ObjectError {
    fn from(source: TryFromIntError) -> Self {
        ObjectError::IntConversionError(source)
    }
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectError::KeyNotFound(key) => write!(f, "Key \"{}\" not found in the object", key),
            ObjectError::WrongType {
                key,
                actual_type,
                wanted_type,
            } => write!(
                f,
                "Key \"{}\" actual type is {:?} and not {:?}",
                key, actual_type, wanted_type
            ),
            ObjectError::IntConversionError(e) => e.fmt(f),
            ObjectError::AddrParseError(e) => e.fmt(f),
            ObjectError::Other(e) => e.fmt(f),
            ObjectError::None => write!(f, "Impossible error was possible after all."),
        }
    }
}

/// Owned and mutable instance of UCL Object.
/// All methods that do not require mutability should be implemented on `ObjectRef` instead.
#[derive(Eq)]
pub struct Object {
    inner: ObjectRef,
}

impl AsRef<ObjectRef> for Object {
    fn as_ref(&self) -> &ObjectRef {
        &self.inner
    }
}

impl Deref for Object {
    type Target = ObjectRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ptr = unsafe { ucl_object_tostring_forced(self.as_ptr()) };
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let as_string = cstr.to_string_lossy().to_string();
        f.debug_struct("Object")
            .field("string_value", &as_string)
            .finish()
    }
}

impl Object {
    pub(crate) fn from_c_ptr(object: *const ucl_object_t) -> Option<Object> {
        ObjectRef::from_c_ptr(object).map(|obj_ref| Object { inner: obj_ref })
    }
}

/// Objects may not actually dropped, but their reference count is decreased.
impl Drop for Object {
    fn drop(&mut self) {
        unsafe {
            if !self.object.is_null() {
                ucl_object_unref(self.object as *mut ucl_object_t);
            }
        }
    }
}

impl Borrow<ObjectRef> for Object {
    fn borrow(&self) -> &ObjectRef {
        &self.inner
    }
}

/// An immutable reference to UCL Object structure.
/// Provides most of the libUCL interface for interacting with parser results.
#[derive(Eq)]
pub struct ObjectRef {
    object: *mut ucl_object_t,
    kind: ucl_type_t,
}

impl ObjectRef {
    /// Return mutable pointer to inner struct.
    pub fn as_mut_ptr(&mut self) -> *mut ucl_object_t {
        self.object
    }
    /// Return const pointer to inner struct.
    pub fn as_ptr(&self) -> *const ucl_object_t {
        self.object
    }

    pub(crate) fn from_c_ptr(object: *const ucl_object_t) -> Option<ObjectRef> {
        if object.is_null() {
            return None;
        }
        let kind = unsafe { ucl_object_type(object) };
        let result = ObjectRef {
            object: object as *mut ucl_object_t,
            kind,
        };
        Some(result)
    }

    /// Perform a deep copy
    pub fn deep_copy(&self) -> Object {
        let ptr = unsafe { ucl_object_copy(self.as_ptr()) };
        Object::from_c_ptr(ptr).expect("Got Object with null ptr")
    }

    /// Returns `true` if this object is a null.
    pub fn is_null(&self) -> bool {
        self.kind == ucl_type_t::UCL_NULL
    }

    /// Returns `true` if this object is an object (think hashmap).
    pub fn is_object(&self) -> bool {
        self.kind == ucl_type_t::UCL_OBJECT
    }

    /// Returns `true` if this object is a string.
    pub fn is_string(&self) -> bool {
        self.kind == ucl_type_t::UCL_STRING
    }

    /// Returns `true` if this object is an integer.
    pub fn is_integer(&self) -> bool {
        self.kind == ucl_type_t::UCL_INT
    }

    /// Returns `true` if this object is a float.
    pub fn is_float(&self) -> bool {
        self.kind == ucl_type_t::UCL_FLOAT
    }

    /// Returns `true` if this object is a boolean type.
    pub fn is_boolean(&self) -> bool {
        self.kind == ucl_type_t::UCL_BOOLEAN
    }

    /// Returns `true` if this object is an array.
    pub fn is_array(&self) -> bool {
        self.kind == ucl_type_t::UCL_ARRAY
    }

    /// Returns `true` if this object is a time/duration.
    pub fn is_time(&self) -> bool {
        self.kind == ucl_type_t::UCL_TIME
    }

    /// Get priority assigned to the object.
    pub fn priority(&self) -> Priority {
        let out = unsafe { ucl_object_get_priority(self.object) };
        Priority::from(out)
    }

    /// Get type/kind of given object
    pub fn kind(&self) -> ucl_type_t {
        self.kind
    }

    /// Get key assigned to the object
    pub fn key(&self) -> Option<String> {
        let c_str = unsafe { ucl_object_key(self.object) };
        utils::to_str(c_str)
    }

    /// Lookup a key within an object with type Object.
    pub fn lookup<K: AsRef<str>>(&self, key: K) -> Option<ObjectRef> {
        if !self.is_object() {
            return None;
        }
        let key = utils::to_c_string(key);
        let obj = unsafe { ucl_object_lookup(self.object, key.as_ptr()) };
        ObjectRef::from_c_ptr(obj as *mut ucl_object_t)
    }

    /// Perform a nested lookup with dot notation.
    pub fn lookup_path<K: AsRef<str>>(&self, path: K) -> Option<ObjectRef> {
        if !self.is_object() {
            return None;
        }
        let key = utils::to_c_string(path);
        let obj = unsafe { ucl_object_lookup_path(self.object, key.as_ptr()) };
        ObjectRef::from_c_ptr(obj as *mut ucl_object_t)
    }
    /// Return string value or None.
    pub fn as_string(&self) -> Option<String> {
        if !self.is_string() {
            return None;
        }
        let mut ptr = MaybeUninit::zeroed();
        let result = unsafe { ucl_object_tostring_safe(self.object, ptr.as_mut_ptr()) };
        if result {
            let ptr = unsafe { ptr.assume_init() };
            utils::to_str(ptr)
        } else {
            None
        }
    }

    /// Return an integer value or None.
    pub fn as_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        let mut ptr = MaybeUninit::zeroed();
        let result = unsafe { ucl_object_toint_safe(self.object, ptr.as_mut_ptr()) };
        if result {
            let ptr = unsafe { ptr.assume_init() };
            Some(ptr)
        } else {
            None
        }
    }

    /// Return a float number of seconds. Only works if object is time.
    pub fn as_time(&self) -> Option<f64> {
        if !self.is_time() {
            return None;
        }
        self.as_f64()
    }

    /// Return a float value or None. This function also works on time object.
    pub fn as_f64(&self) -> Option<f64> {
        if !(self.is_float() || self.is_time()) {
            return None;
        }
        let mut ptr = MaybeUninit::zeroed();
        let result = unsafe { ucl_object_todouble_safe(self.object, ptr.as_mut_ptr()) };
        if result {
            let ptr = unsafe { ptr.assume_init() };
            Some(ptr)
        } else {
            None
        }
    }

    /// Return a boolean value or None.
    pub fn as_bool(&self) -> Option<bool> {
        if !self.is_boolean() {
            return None;
        }
        let mut ptr = MaybeUninit::zeroed();
        let result = unsafe { ucl_object_toboolean_safe(self.object, ptr.as_mut_ptr()) };
        if result {
            let ptr = unsafe { ptr.assume_init() };
            Some(ptr)
        } else {
            None
        }
    }

    /// Return `()` or None.
    pub fn as_null(&self) -> Option<()> {
        if !self.is_null() {
            return None;
        }
        Some(())
    }

    /// Preferred way to construct an iterator. Items returned by this iterator are always `ObjectRef`.
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}

impl From<i64> for Object {
    fn from(source: i64) -> Self {
        let ptr = unsafe { ucl_object_fromint(source) };
        Object::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl From<f64> for Object {
    fn from(source: f64) -> Self {
        let ptr = unsafe { ucl_object_fromdouble(source) };
        Object::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl From<bool> for Object {
    fn from(source: bool) -> Self {
        let ptr = unsafe { ucl_object_frombool(source) };
        Object::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}
impl From<&str> for Object {
    fn from(source: &str) -> Self {
        let cstring = utils::to_c_string(source);
        let ptr = unsafe { ucl_object_fromstring(cstring.as_ptr()) };
        Object::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl FromObject<ObjectRef> for i64 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_i64() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for u64 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for i32 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for u32 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for i16 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for u16 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for i8 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for u8 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(val) = value.as_i64() {
            val.try_into().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for f64 {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_f64() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_FLOAT,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for bool {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_bool() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_BOOLEAN,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for () {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if value.is_null() {
            Ok(())
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_NULL,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for String {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_string() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_STRING,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for PathBuf {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_string() {
            Ok(ret.into())
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_STRING,
            };
            Err(err)
        }
    }
}

impl FromObject<ObjectRef> for SocketAddr {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(ret) = value.as_string() {
            ret.parse().map_err(ObjectError::from)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_STRING,
            };
            Err(err)
        }
    }
}
impl<T> FromObject<ObjectRef> for Vec<T>
where
    T: FromObject<ObjectRef>,
{
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        let ret = value.iter()
            .map(T::try_from)
            .collect::<Vec<Result<T, ObjectError>>>();
        if let Some(Err(err)) = ret.iter().find(|e| e.is_err()) {
            Err(err.clone())
        } else {
            let list = ret.into_iter().filter_map(|e| e.ok() ).collect();
            Ok(list)
        }
    }
}

impl<T> FromObject<ObjectRef> for Option<T>
where
    T: FromObject<ObjectRef>,
{
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        (T::try_from(value)).map(Some)
    }
}

impl<T, S> FromObject<ObjectRef> for HashMap<String, T, S>
where
    T: FromObject<ObjectRef> + Clone,
    S: BuildHasher + Default,
{
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if ucl_type_t::UCL_OBJECT != value.kind {
            return Err(ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_OBJECT,
            });
        }
        let as_entries: Vec<(String, Result<T, ObjectError>)> = value
            .iter()
            .map(|obj| {
                (
                    obj.key().expect("Object without key!"),
                    FromObject::try_from(obj),
                )
            })
            .collect();

        if let Some((_, Err(e))) = as_entries.iter().find(|(_key, result)| result.is_err()) {
            Err(e.clone())
        } else {
            Ok(as_entries
                .iter()
                .cloned()
                .map(|(key, result)| (key, result.unwrap()))
                .collect())
        }
    }
}

impl FromObject<ObjectRef> for Duration {
    fn try_from(value: ObjectRef) -> Result<Self, ObjectError> {
        if let Some(seconds) = value.as_time() {
            Ok(Duration::from_secs_f64(seconds))
        } else {
            return Err(ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_TIME,
            });
        }
    }
}

impl fmt::Debug for ObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ptr = unsafe { ucl_object_tostring_forced(self.as_ptr()) };
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let as_string = cstr.to_string_lossy().to_string();
        f.debug_struct("ObjectRef")
            .field("string_value", &as_string)
            .finish()
    }
}

impl ToOwned for ObjectRef {
    type Owned = Object;

    fn to_owned(&self) -> Self::Owned {
        let ptr = unsafe { ucl_object_ref(self.as_ptr()) };
        Object::from_c_ptr(ptr).expect("Got ObjectRef with null ptr")
    }
}

impl PartialEq for ObjectRef {
    fn eq(&self, other: &Self) -> bool {
        let cmp = unsafe { ucl_object_compare(self.as_ptr(), other.as_ptr() )};
        cmp == 0
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        let ptr = unsafe { ucl_object_ref(self.as_ptr()) };
        Object::from_c_ptr(ptr).expect("Got ObjectRef with null ptr")
    }
}

impl PartialOrd for ObjectRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = unsafe { ucl_object_compare(self.as_ptr(), other.as_ptr() )};
        match cmp {
            cmp if cmp == 0 => Some(Ordering::Equal),
            cmp if cmp < 0 => Some(Ordering::Less),
            cmp if cmp > 0 => Some(Ordering::Greater),
            _ => unreachable!(),
        }
    }
}

impl Ord for ObjectRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eq() {
        let left = Object::from(1);
        let right = Object::from(1);
        let right_but_wrong = Object::from(2);

        assert_eq!(left, right);
        assert_ne!(left, right_but_wrong);
    }

    #[test]
    fn deep_copy() {
        let left = Object::from(1);
        let right = left.deep_copy();
        assert_eq!(left, right);
        assert_ne!(left.as_ptr(), right.as_ptr());
    }

    #[test]
    fn order_good() {
        let left = Object::from(1);
        let right = Object::from(2);

        assert!(left < right);
    }

    #[test]
    fn order_int_and_float() {

        let left = Object::from(1);
        let right = Object::from(1.5);

        assert!(left < right);
    }

    #[test]
    fn order_wtf() {
        let left = Object::from("a string?");
        let right = Object::from(2);

        assert_ne!(left, right);
    }
}
