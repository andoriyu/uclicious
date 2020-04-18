use crate::traits::{unpack_closure, VariableHandler};
use libucl_bind::ucl_variable_handler;
use std::ffi::c_void;
use std::os::raw::c_uchar;
use std::ptr::slice_from_raw_parts;

pub struct EnvVariableHandler {
    closure:
        Box<dyn FnMut(*const c_uchar, usize, *mut *mut c_uchar, *mut usize, *mut bool) -> bool>,
}

impl EnvVariableHandler {
    fn with_prefix(prefix: String) -> Self {
        let closure = move |data: *const ::std::os::raw::c_uchar,
                            len: usize,
                            replace: *mut *mut ::std::os::raw::c_uchar,
                            replace_len: *mut usize,
                            need_free: *mut bool| {
            let var = unsafe {
                let slice = slice_from_raw_parts(data, len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };

            if var.starts_with(&prefix) {
                if let Ok(mut value) = std::env::var(var) {
                    let bytes = unsafe { value.as_bytes_mut() };
                    unsafe {
                        *need_free = false;
                        *replace = bytes.as_mut_ptr();
                        *replace_len = bytes.len();
                    }
                    return true;
                }
            }
            false
        };
        EnvVariableHandler {
            closure: Box::new(closure),
        }
    }
}
impl Default for EnvVariableHandler {
    fn default() -> Self {
        Self::with_prefix(String::from("ENV_"))
    }
}

impl VariableHandler for EnvVariableHandler {
    fn handle(
        &mut self,
        ptr: *const u8,
        len: usize,
        dst: *mut *mut u8,
        dst_len: *mut usize,
        needs_free: *mut bool,
    ) -> bool {
        self.closure.handle(ptr, len, dst, dst_len, needs_free)
    }

    fn get_fn_ptr_and_data(&mut self) -> (*mut c_void, ucl_variable_handler) {
        unsafe { unpack_closure(&mut self.closure) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::VariableHandler;
    use crate::{Parser, Priority, DEFAULT_DUPLICATE_STRATEGY};

    #[test]
    fn basic_env_var_handler() {
        let mut handler = EnvVariableHandler::default();
        let (state, callback) = handler.get_fn_ptr_and_data();

        let good_var = "ENV_RZZYIBBEBD";
        let bad_var = "ENV_AGBDMXLAAH";

        std::env::set_var(good_var, "yes");
        std::env::remove_var(bad_var);
        std::env::set_var("RZZYIBBEBD", "yes");

        let input = r#"
        good = "${ENV_RZZYIBBEBD}"
        bad = "${ENV_AGBDMXLAAH}"
        also_bad = "${RZZYIBBEBD}"
        "#;

        let mut parser = Parser::default();
        unsafe { parser.set_variables_handler_raw(callback, state); }
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let root = parser.get_object().unwrap();

        let good = root.lookup("good").unwrap().as_string().unwrap();
        assert_eq!("yes", &good);

        let bad = root.lookup("bad").unwrap().as_string().unwrap();
        assert_eq!("${ENV_AGBDMXLAAH}", bad);

        let also_bad = root.lookup("also_bad").unwrap().as_string().unwrap();
        assert_eq!("${RZZYIBBEBD}", also_bad);
    }
}
