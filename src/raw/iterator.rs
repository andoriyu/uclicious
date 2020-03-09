use super::object::ObjectRef;
use libucl_bind::{ucl_object_iterate_new, ucl_object_iterate_safe, ucl_object_iterate_free};

pub struct IterMut<'data> {
    object: &'data mut ObjectRef,
    inner: libucl_bind::ucl_object_iter_t
}

impl<'data> IterMut<'data> {
    pub fn new(object: &'data mut ObjectRef) -> Self {
        let inner = unsafe {
          ucl_object_iterate_new(object.as_mut_ptr())
        };
        IterMut {
            object,
            inner
        }
    }
}

impl<'data> Iterator for IterMut<'data> {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        // if it's not an array or iterator failed to initialize then bail early.
        if !(self.object.is_array() || self.object.is_object()) || self.inner.is_null() {
            return None;
        }
        let obj_ptr = unsafe {
            ucl_object_iterate_safe(self.inner, true)
        };

        ObjectRef::from_c_ptr(obj_ptr)
    }
}

impl<'data> Drop for IterMut<'data> {
    fn drop(&mut self) {
        unsafe { ucl_object_iterate_free(self.inner); }
    }
}

pub struct Iter<'data> {
    object: &'data ObjectRef,
    inner: libucl_bind::ucl_object_iter_t
}

impl<'data> Iter<'data> {
    pub fn new(object: &'data ObjectRef) -> Self {
        let inner = unsafe {
            ucl_object_iterate_new(object.as_ptr())
        };
        Iter {
            object,
            inner
        }
    }
}

impl<'data> Iterator for Iter<'data> {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        // if it's not an array or iterator failed to initialize then bail early.
        if !(self.object.is_array() || self.object.is_object()) || self.inner.is_null() {
            return None;
        }
        let obj_ptr = unsafe {
            ucl_object_iterate_safe(self.inner, true)
        };

        ObjectRef::from_c_ptr(obj_ptr)
    }
}

impl<'data> Drop for Iter<'data> {
    fn drop(&mut self) {
        unsafe { ucl_object_iterate_free(self.inner); }
    }
}

pub struct IntoIter {
    object: ObjectRef,
    inner: libucl_bind::ucl_object_iter_t
}

impl<'data> IntoIter {
    pub fn new(object: ObjectRef) -> Self {
        let inner = unsafe {
            ucl_object_iterate_new(object.as_ptr())
        };
        IntoIter {
            object,
            inner
        }
    }
}

impl Iterator for IntoIter {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        // if it's not an array or iterator failed to initialize then bail early.
        if !(self.object.is_array() || self.object.is_object()) || self.inner.is_null() {
            return None;
        }
        let obj_ptr = unsafe {
            ucl_object_iterate_safe(self.inner, true)
        };

        ObjectRef::from_c_ptr(obj_ptr)
    }
}

impl Drop for IntoIter {
    fn drop(&mut self) {
        unsafe { ucl_object_iterate_free(self.inner); }
    }
}

