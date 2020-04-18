use crate::traits::{unpack_closure, VariableHandler};
use libucl_bind::ucl_variable_handler;
use std::cell::RefCell;
use std::ffi::c_void;
use std::os::raw::c_uchar;
use std::rc::Rc;

pub struct CompoundHandler {
    handlers: Rc<RefCell<Vec<Box<dyn VariableHandler>>>>,
    closure:
        Box<dyn FnMut(*const c_uchar, usize, *mut *mut c_uchar, *mut usize, *mut bool) -> bool>,
}

impl Default for CompoundHandler {
    fn default() -> Self {
        let handlers: Rc<RefCell<Vec<Box<dyn VariableHandler>>>> = Default::default();
        let handlers_rc = handlers.clone();
        let closure = move |data: *const ::std::os::raw::c_uchar,
                            len: usize,
                            replace: *mut *mut ::std::os::raw::c_uchar,
                            replace_len: *mut usize,
                            need_free: *mut bool| {
            let mut handlers = handlers_rc.borrow_mut();
            let mut found = false;

            for handler in handlers.iter_mut() {
                if handler.handle(data, len, replace, replace_len, need_free) {
                    found = true;
                    break;
                } else {
                }
            }
            found
        };

        CompoundHandler {
            handlers,
            closure: Box::new(closure),
        }
    }
}

impl CompoundHandler {
    pub fn register_handler(&mut self, handler: Box<dyn VariableHandler>) -> &mut Self {
        {
            let mut handlers = self.handlers.borrow_mut();
            handlers.push(handler);
        }
        self
    }
}

impl VariableHandler for CompoundHandler {
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
    use crate::{Parser, Priority, DEFAULT_DUPLICATE_STRATEGY};
    use std::ptr::slice_from_raw_parts;

    #[test]
    fn compound_handler() {
        let www = |data: *const ::std::os::raw::c_uchar,
                   len: usize,
                   replace: *mut *mut ::std::os::raw::c_uchar,
                   replace_len: *mut usize,
                   need_free: *mut bool| {
            let var = unsafe {
                let slice = slice_from_raw_parts(data, len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };
            if var.eq("WWW") {
                let test = "asd";
                let size = test.as_bytes().len();
                unsafe {
                    *replace = libc::malloc(size).cast();
                    *replace_len = size;
                    test.as_bytes()
                        .as_ptr()
                        .copy_to_nonoverlapping(*replace, size);
                    *need_free = true;
                }
                true
            } else {
                false
            }
        };

        let zzz = |data: *const ::std::os::raw::c_uchar,
                   len: usize,
                   replace: *mut *mut ::std::os::raw::c_uchar,
                   replace_len: *mut usize,
                   need_free: *mut bool| {
            let var = unsafe {
                let slice = slice_from_raw_parts(data, len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };
            if var.eq("ZZZ") {
                let test = "dsa";
                let size = test.as_bytes().len();
                unsafe {
                    *replace = libc::malloc(size).cast();
                    *replace_len = size;
                    test.as_bytes()
                        .as_ptr()
                        .copy_to_nonoverlapping(*replace, size);
                    *need_free = true;
                }
                true
            } else {
                false
            }
        };

        let mut compound_handler = CompoundHandler::default();
        compound_handler.register_handler(Box::new(www));
        compound_handler.register_handler(Box::new(zzz));
        let (state, callback) = compound_handler.get_fn_ptr_and_data();

        let input = r#"
        key2 = "${ZZZ}"
        key1 = "${WWW}"
        "#;
        let mut parser = Parser::default();
        unsafe {
            parser.set_variables_handler_raw(callback, state);
        }
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let root = parser.get_object().unwrap();

        let looked_up_object1 = root.lookup("key1").unwrap();
        dbg!(&looked_up_object1);
        let object1 = looked_up_object1.as_string().unwrap();
        assert_eq!("asd", object1.as_str());

        let looked_up_object2 = root.lookup("key2").unwrap();
        dbg!(&looked_up_object2);
        let object2 = looked_up_object2.as_string().unwrap();
        assert_eq!("dsa", object2.as_str());
    }
}
