use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub(crate) fn to_str(cstring: *const c_char) -> Option<String> {
    if cstring.is_null() {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(cstring) };
    Some(c_str.to_string_lossy().into_owned())
}

pub(crate) fn to_c_string<S: AsRef<str>>(str: S) -> CString {
    CString::new(str.as_ref().as_bytes()).expect("Path cannot contain null character")
}


#[cfg(test)]
mod test {
    use crate::raw::utils::{to_str, to_c_string};

    #[test]
    fn nullpointer() {
        let np = std::ptr::null();
        let result = to_str(np);
        assert!(result.is_none());
    }

    #[test]
    #[should_panic]
    fn to_c_string_with_null() {
        let input = "abc\0d";
        let _ = to_c_string(input);
    }
}