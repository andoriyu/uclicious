//! Various traits that help you reduce boilerplate while using uclicious.
use crate::ObjectError;
use libucl_bind::ucl_variable_handler;
use std::os::raw::{c_uchar, c_void};

/// Implement this trait on your types in order for automatic derive to work. This is a copy of `TryFrom`.
pub trait FromObject<T>: Sized {
    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, ObjectError>;
}

pub trait TryInto<T>: Sized {
    fn try_into(self) -> Result<T, ObjectError>;
}

impl<T, U> TryInto<U> for T
where
    U: FromObject<T>,
{
    fn try_into(self) -> Result<U, ObjectError> {
        U::try_from(self)
    }
}

/// A safe-ish interface that can be used as var handler. It's not implemented for a lot of things right now.
pub trait VariableHandler {
    fn handle(
        &mut self,
        ptr: *const ::std::os::raw::c_uchar,
        len: usize,
        dst: *mut *mut ::std::os::raw::c_uchar,
        dst_len: *mut usize,
        needs_free: *mut bool,
    ) -> bool;
    fn get_fn_ptr_and_data(&mut self) -> (*mut c_void, ucl_variable_handler);
}

/// Return the length of the variable name within the buffer libucl passes to a
/// variable handler.
///
/// libucl 0.9.x invokes the handler with two different conventions depending on
/// the pass:
///
/// * the sizing pass (`ucl_check_variable_safe`) passes the isolated variable
///   name, e.g. `("FOO", 3)` for `${FOO}`;
/// * the expansion pass (`ucl_expand_single_variable`) passes a pointer into the
///   *remaining* configuration buffer with the length of that remainder, e.g.
///   `("FOO}…", 4+)`.
///
/// Handlers want the variable name in both cases, so this returns the number of
/// leading bytes up to (but excluding) the first `}` — or the whole `len` when
/// there is none. It is a no-op for the already-isolated sizing-pass form, so it
/// is safe to apply unconditionally. Custom raw handlers registered via
/// [`set_variables_handler_raw`](crate::Parser::set_variables_handler_raw)
/// should call this to obtain the name; the safe [`VariableHandler`] path
/// applies it automatically.
///
/// # Safety
///
/// `ptr` must be valid for reads of `len` bytes.
pub unsafe fn variable_name_len(ptr: *const c_uchar, len: usize) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    match slice.iter().position(|&b| b == b'}') {
        Some(pos) => pos,
        None => len,
    }
}

/// Unpack closure into a data (context) and function pointer. Copy pasted from FFI guide and adopted for this specific use case.
///
/// # Safety
///
/// Caller need to ensure that closure lives as log as the pointer does.
pub(crate) unsafe fn unpack_closure<F>(closure: &mut F) -> (*mut c_void, ucl_variable_handler)
where
    F: FnMut(*const c_uchar, usize, *mut *mut c_uchar, *mut usize, *mut bool) -> bool,
{
    extern "C" fn trampoline<F>(
        ptr: *const ::std::os::raw::c_uchar,
        len: usize,
        dst: *mut *mut ::std::os::raw::c_uchar,
        dst_len: *mut usize,
        needs_free: *mut bool,
        data: *mut c_void,
    ) -> bool
    where
        F: FnMut(*const c_uchar, usize, *mut *mut c_uchar, *mut usize, *mut bool) -> bool,
    {
        // Normalise libucl 0.9.x's expansion-pass buffer (which includes the
        // closing brace and trailing bytes) down to the variable name so safe
        // handlers see a consistent `(ptr, name_len)` across both passes.
        let name_len = unsafe { variable_name_len(ptr, len) };
        let closure: &mut F = unsafe { &mut *(data as *mut F) };
        (*closure)(ptr, name_len, dst, dst_len, needs_free)
    }
    (closure as *mut F as *mut c_void, Some(trampoline::<F>))
}

impl<F> VariableHandler for F
where
    F: FnMut(*const c_uchar, usize, *mut *mut c_uchar, *mut usize, *mut bool) -> bool,
{
    fn handle(
        &mut self,
        ptr: *const u8,
        len: usize,
        dst: *mut *mut u8,
        dst_len: *mut usize,
        needs_free: *mut bool,
    ) -> bool {
        self(ptr, len, dst, dst_len, needs_free)
    }

    fn get_fn_ptr_and_data(&mut self) -> (*mut c_void, ucl_variable_handler) {
        unsafe { unpack_closure(self) }
    }
}
