//! Iterators for `ObjectRef`. Currently, none of them supports implicit arrays. That *may* change in the future, but right not right now.

use super::object::ObjectRef;
use libucl_bind::{ucl_object_iterate_free, ucl_object_iterate_new, ucl_object_iterate_safe};

pub struct Iter<'data> {
    object: &'data ObjectRef,
    inner: libucl_bind::ucl_object_iter_t,
}

impl<'data> Iter<'data> {
    pub fn new(object: &'data ObjectRef) -> Self {
        let inner = unsafe { ucl_object_iterate_new(object.as_ptr()) };
        Iter { object, inner }
    }
}

pub struct IntoIter {
    object: ObjectRef,
    inner: libucl_bind::ucl_object_iter_t,
}

impl IntoIter {
    pub fn new(object: ObjectRef) -> Self {
        let inner = unsafe { ucl_object_iterate_new(object.as_ptr()) };
        IntoIter { object, inner }
    }
}

impl<'data> Iterator for Iter<'data> {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        iterate(&self.object, self.inner)
    }
}

impl Iterator for IntoIter {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        iterate(&self.object, self.inner)
    }
}

impl<'data> Drop for Iter<'data> {
    fn drop(&mut self) {
        unsafe {
            ucl_object_iterate_free(self.inner);
        }
    }
}

impl Drop for IntoIter {
    fn drop(&mut self) {
        unsafe {
            ucl_object_iterate_free(self.inner);
        }
    }
}

impl<'data> IntoIterator for &'data ObjectRef {
    type Item = ObjectRef;
    type IntoIter = Iter<'data>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl IntoIterator for ObjectRef {
    type Item = ObjectRef;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}


fn iterate(object: &ObjectRef, iterator: libucl_bind::ucl_object_iter_t) -> Option<ObjectRef> {
    // bail early if it's not an array or iterator didn't initialize.
    if !(object.is_array() || object.is_object()) || iterator.is_null() {
        return None;
    }
    let obj_ptr = unsafe { ucl_object_iterate_safe(iterator, true) };

    ObjectRef::from_c_ptr(obj_ptr)
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn basic_array() {
        let mut parser = Parser::default();
        let input = r#"array = [1,2,3]"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("array").unwrap();

        let actual: Vec<i64> = lookup_result.iter().map(|obj| obj.as_i64().unwrap()).collect();

        let expected = vec![1i64, 2, 3];

        assert_eq!(expected, actual);
    }

    #[test]
    fn not_an_array() {
        let mut parser = Parser::default();
        let input = r#"not_an_array = 1"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("not_an_array").unwrap();

        let next = lookup_result.iter().next();

        assert!(next.is_none());
    }

    #[test]
    fn into_iter_borrowed() {
        let mut parser = Parser::default();
        let input = r#"array = [1,2,3]"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("array").unwrap();

        let mut count = 0;

        for _obj in &lookup_result {
            count += 1;
        }
        assert_eq!(3, count);
    }

    #[test]
    fn into_iter_owned() {
        let mut parser = Parser::default();
        let input = r#"array = [1,2,3]"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("array").unwrap();

        let mut count = 0;

        for _obj in lookup_result {
            count += 1;
        }
        assert_eq!(3, count);
    }

    #[test]
    fn iter_object() {

        let mut parser = Parser::default();
        let input = r#"dict = {
            a = 1,
            b = 2,
        }"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("dict").unwrap();

        let mut iter = lookup_result.iter();
        {
            let next = iter.next().unwrap();
            assert_eq!(Some("a".to_string()), next.key());
            assert_eq!(Some(1), next.as_i64());
        }
        {
            let next = iter.next().unwrap();
            assert_eq!(Some("b".to_string()), next.key());
            assert_eq!(Some(2), next.as_i64());
        }

        assert!(iter.next().is_none());
    }
}