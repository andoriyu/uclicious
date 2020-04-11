//! Clone of TryFrom from std. A hack around std's blanket implementations.
use crate::ObjectError;
use libucl_bind::ucl_variable_handler;
use std::os::raw::{c_uchar, c_void};

/// Implement this trait on your types in order for automatic derive to work.
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

pub unsafe fn unpack_closure<F>(closure: &mut F) -> (*mut c_void, ucl_variable_handler)
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
        let closure: &mut F = unsafe { &mut *(data as *mut F) };
        (*closure)(ptr, len, dst, dst_len, needs_free)
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
