use libucl_bind::{ucl_object_t, ucl_type_t, ucl_object_type, ucl_object_lookup, ucl_object_unref, ucl_object_tostring, ucl_object_toint, ucl_object_todouble, ucl_object_get_priority, ucl_object_toboolean, ucl_object_key, ucl_object_fromint, ucl_object_fromdouble, ucl_object_frombool, ucl_object_lookup_path, ucl_object_tostring_forced, ucl_object_tostring_safe, ucl_object_toint_safe};
use crate::raw::{utils, Priority};
use std::error::Error;
use std::fmt;
use std::convert::TryFrom;
use crate::raw::iterator::{IterMut, Iter, IntoIter};
use bitflags::_core::fmt::Formatter;
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::mem::MaybeUninit;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ObjectError {
    KeyNotFound(String),
    WrongType { key: String, actual_type: ucl_type_t, wanted_type: ucl_type_t},
}

impl Error for ObjectError {}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectError::KeyNotFound(key) => {
                write!(f, "Key \"{}\" not found in the object", key)
            },
            ObjectError::WrongType {key, actual_type,wanted_type} => {
                write!(f, "Key \"{}\" actual type is {:?} and not {:?}", key, actual_type, wanted_type)
            },
        }
    }
}


pub struct  Object {
    inner: ObjectRef
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

impl Object {
    pub(crate) fn from_c_ptr(object: *const ucl_object_t) -> Option<Object> {
        ObjectRef::from_c_ptr(object)
            .map(|obj_ref| Object {
                inner: obj_ref
            })
    }
}

/// UCL Object structure. T
pub struct ObjectRef {
    object: *mut ucl_object_t,
    kind: ucl_type_t,
}

impl ObjectRef {
    pub fn as_mut_ptr(&mut self) -> *mut ucl_object_t {
        self.object
    }

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
            kind
        };
        Some(result)
    }

    pub fn is_null(&self) -> bool {
        self.kind == ucl_type_t::UCL_NULL
    }

    pub fn is_object(&self) -> bool {
        self.kind == ucl_type_t::UCL_OBJECT
    }

    pub fn is_string(&self) -> bool {
        self.kind == ucl_type_t::UCL_STRING
    }

    pub fn is_integer(&self) -> bool {
        self.kind == ucl_type_t::UCL_INT
    }

    pub fn is_float(&self) -> bool {
        self.kind == ucl_type_t::UCL_FLOAT
    }

    pub fn is_boolean(&self) -> bool {
        self.kind == ucl_type_t::UCL_BOOLEAN
    }

    pub fn is_array(&self) -> bool {
        self.kind == ucl_type_t::UCL_ARRAY
    }

    /// Get priority assigned to the object.
    pub fn priority(&self) -> Priority {
        let out = unsafe {
            ucl_object_get_priority(self.object)
        };
        Priority::from(out)
    }

    /// Get type/kind of given object
    pub fn kind(&self) -> ucl_type_t {
        self.kind
    }

    /// Get key assigned to the object
    pub fn key(&self) -> Option<String> {
        let c_str = unsafe {
            ucl_object_key(self.object)
        };
        utils::to_str(c_str)
    }

    pub fn lookup<K: AsRef<str>>(&self, key: K) -> Option<Object> {
        if !self.is_object() {
            return None;
        }
        let key = utils::to_c_string(key);

        let obj = unsafe {
            ucl_object_lookup(self.object, key.as_ptr())
        };

        Object::from_c_ptr(obj as *mut ucl_object_t)
    }

    pub fn lookup_path<K: AsRef<str>>(&self, path: K) -> Option<ObjectRef> {
        if !self.is_object() {
            return None;
        }
        let key = utils::to_c_string(path);

        let obj = unsafe {
            ucl_object_lookup_path(self.object, key.as_ptr())
        };

        ObjectRef::from_c_ptr(obj as *mut ucl_object_t)
    }
    /// Return string value
    pub fn as_string(&self) -> Option<String> {

        if !self.is_string() { return None }

        let mut ptr = MaybeUninit::zeroed();
        let result = unsafe {
            ucl_object_tostring_safe(self.object, ptr.as_mut_ptr())
        };

        if result {
            let ptr = unsafe { ptr.assume_init() };
            utils::to_str(ptr)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None
        }

        let mut ptr = MaybeUninit::zeroed();

        let result = unsafe {
            ucl_object_toint_safe(self.object, ptr.as_mut_ptr())
        };
        if result {
            let ptr = unsafe { ptr.assume_init() };
            Some(ptr)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if !self.is_float() {
            return None;
        }

        let out = unsafe {
            ucl_object_todouble(self.object)
        };
        Some(out)
    }

    pub fn as_bool(&self) -> Option<bool> {
        if !self.is_boolean() {
            return None;
        }
        let out = unsafe {
            ucl_object_toboolean(self.object)
        };
        Some(out)
    }

    pub fn as_null(&self) -> Option<()> {
        if !self.is_boolean() {
            return None;
        }
        Some(())
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        unsafe {
            if !self.object.is_null() { ucl_object_unref(self.object as *mut ucl_object_t); }
        }
    }
}

impl From<i64> for ObjectRef {
    fn from(source: i64) -> Self {
        let ptr = unsafe {
            ucl_object_fromint(source)
        };
        ObjectRef::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl From<f64> for ObjectRef {
    fn from(source: f64) -> Self {
        let ptr = unsafe {
            ucl_object_fromdouble(source)
        };
        ObjectRef::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl From<bool> for ObjectRef {
    fn from(source: bool) -> Self {
        let ptr = unsafe {
            ucl_object_frombool(source)
        };
        ObjectRef::from_c_ptr(ptr).expect("Failed to construct an object.")
    }
}

impl TryFrom<ObjectRef> for i64 {
    type Error = ObjectError;

    fn try_from(value: ObjectRef) -> Result<Self, Self::Error> {
        if let Some(ret) = value.as_i64() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_INT
            };
            Err(err)
        }
    }
}

impl TryFrom<ObjectRef> for f64 {
    type Error = ObjectError;

    fn try_from(value: ObjectRef) -> Result<Self, Self::Error> {
        if let Some(ret) = value.as_f64() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_FLOAT
            };
            Err(err)
        }
    }
}

impl TryFrom<ObjectRef> for bool {
    type Error = ObjectError;

    fn try_from(value: ObjectRef) -> Result<Self, Self::Error> {
        if let Some(ret) = value.as_bool() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_BOOLEAN
            };
            Err(err)
        }
    }
}

impl TryFrom<ObjectRef> for () {
    type Error = ObjectError;

    fn try_from(value: ObjectRef) -> Result<Self, Self::Error> {
        if let Some(ret) = value.as_null() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_NULL
            };
            Err(err)
        }
    }
}

impl TryFrom<ObjectRef> for String {
    type Error = ObjectError;

    fn try_from(value: ObjectRef) -> Result<Self, Self::Error> {
        if let Some(ret) = value.as_string() {
            Ok(ret)
        } else {
            let err = ObjectError::WrongType {
                key: value.key().unwrap_or_default(),
                actual_type: value.kind,
                wanted_type: ucl_type_t::UCL_STRING
            };
            Err(err)
        }
    }
}

impl fmt::Debug for ObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ptr = unsafe {
            ucl_object_tostring_forced(self.as_ptr())
        };
        let cstr = unsafe {CStr::from_ptr(ptr)};
        write!(f, "{:?}", cstr)
    }
}

impl<'data> IntoIterator for &'data mut ObjectRef {
    type Item = ObjectRef;
    type IntoIter = IterMut<'data>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}
impl<'data> IntoIterator for &'data ObjectRef {
    type Item = ObjectRef;
    type IntoIter = Iter<'data>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
impl<'data> IntoIterator for ObjectRef {
    type Item = ObjectRef;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
