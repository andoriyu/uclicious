//! Output of the parser.
use crate::raw::iterator::Iter;
use crate::raw::{utils, Priority};
use crate::traits::FromObject;
use bitflags::_core::borrow::Borrow;
use bitflags::_core::convert::Infallible;
use bitflags::_core::fmt::Formatter;
use libucl_bind::{
    ucl_object_frombool, ucl_object_fromdouble, ucl_object_fromint, ucl_object_fromstring,
    ucl_object_get_priority, ucl_object_key, ucl_object_lookup, ucl_object_lookup_path,
    ucl_object_ref, ucl_object_t, ucl_object_toboolean_safe, ucl_object_todouble_safe,
    ucl_object_toint_safe, ucl_object_tostring_forced, ucl_object_tostring_safe, ucl_object_type,
    ucl_object_unref, ucl_type_t,
};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::mem::MaybeUninit;
use std::net::{AddrParseError, SocketAddr};
use std::num::TryFromIntError;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use bitflags::_core::hash::BuildHasher;

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
    /// Not an error, but required for some conversions.
    None,
}

impl Error for ObjectError {}

impl ObjectError {
    pub fn boxed(self) -> Box<ObjectError> {
        Box::new(self)
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
            ObjectError::None => write!(f, "Impossible error was possible after all."),
        }
    }
}

/// Owned and mutable instance of UCL Object.
/// All methods that do not require mutability should be implemented on `ObjectRef` instead.
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

    /// Return a float value or None.
    pub fn as_f64(&self) -> Option<f64> {
        if !self.is_float() {
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
        if ucl_type_t::UCL_ARRAY == value.kind {
            let ret: Vec<Result<T, ObjectError>> = value.iter().map(T::try_from).collect();
            if let Some(Err(e)) = ret.iter().find(|e| e.is_err()) {
                Err(e.clone())
            } else {
                let list: Vec<T> = ret.into_iter().map(Result::unwrap).collect();
                Ok(list)
            }
        } else {
            FromObject::try_from(value).map(|e| vec![e])
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

impl<T,S> FromObject<ObjectRef> for HashMap<String, T, S>
where
    T: FromObject<ObjectRef> + Clone,
    S: BuildHasher + Default
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
