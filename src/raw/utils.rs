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
