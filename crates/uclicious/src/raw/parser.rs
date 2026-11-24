//! Safe wrapper for libUCL parser.
//! ## Usage
//! ```no_run
//! use uclicious::*;
//! use std::path::PathBuf;
//! let mut parser = Parser::default();
//! let input = r#"
//! test_string = "no scope"
//! "#;
//! let jails_conf = PathBuf::from("/etc/jails.conf");
//! parser.add_chunk_full("enabled = false", Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
//! parser.add_file_full(&jails_conf, Priority::new(15), DEFAULT_DUPLICATE_STRATEGY).unwrap();
//! parser.set_filevars(&jails_conf, true);
//!
//! let result = parser.get_object().unwrap();
//! ```
use crate::raw::{DuplicateStrategy, Priority};
use libucl_bind::{
    ucl_parse_type, ucl_parser, ucl_parser_add_chunk_full, ucl_parser_add_fd_full,
    ucl_parser_add_file_full, ucl_parser_free, ucl_parser_get_error, ucl_parser_get_error_code,
    ucl_parser_get_object, ucl_parser_new, ucl_parser_register_variable, ucl_parser_set_filevars,
    ucl_parser_set_variables_handler, ucl_variable_handler,
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use super::{utils, ParserFlags, DEFAULT_PARSER_FLAG};
use crate::error;
use crate::raw::object::Object;
use crate::traits::VariableHandler;
use std::fmt;
use std::path::Path;

/// Raw parser object.
pub struct Parser {
    parser: *mut ucl_parser,
    flags: ParserFlags,
    object: Option<Object>,
    var_handler: Option<Box<dyn VariableHandler>>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::with_flags(DEFAULT_PARSER_FLAG)
    }
}

impl Parser {
    fn get_error(&mut self) -> error::UclError {
        let err = unsafe { ucl_parser_get_error_code(self.parser) };
        let desc = unsafe { ucl_parser_get_error(self.parser) };

        error::UclErrorType::from_code(err, utils::to_str(desc).unwrap())
    }

    /// Create a new parser with given option flags.
    pub fn with_flags(flags: ParserFlags) -> Self {
        Parser {
            parser: unsafe { ucl_parser_new(flags.0 as i32) },
            flags,
            object: None,
            var_handler: None,
        }
    }

    /// Add a chunk of text to the parser. String must:
    /// - not have `\0` character;
    /// - must be valid UCL object;
    pub fn add_chunk_full<C: AsRef<str>>(
        &mut self,
        chunk: C,
        priority: Priority,
        strategy: DuplicateStrategy,
    ) -> Result<(), error::UclError> {
        let chunk = chunk.as_ref();
        let result = unsafe {
            ucl_parser_add_chunk_full(
                self.parser,
                chunk.as_ptr(),
                chunk.len(),
                priority.as_c_uint(),
                strategy,
                ucl_parse_type::UCL_PARSE_AUTO,
            )
        };
        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    /// Add a file by a file path to the parser. This function uses mmap call to load file, therefore, it should not be shrunk during parsing.
    pub fn add_file_full<F: AsRef<Path>>(
        &mut self,
        file: F,
        priority: Priority,
        strategy: DuplicateStrategy,
    ) -> Result<(), error::UclError> {
        let file_path = utils::to_c_string(file.as_ref().to_string_lossy());
        let result = unsafe {
            ucl_parser_add_file_full(
                self.parser,
                file_path.as_ptr(),
                priority.as_c_uint(),
                strategy,
                ucl_parse_type::UCL_PARSE_AUTO,
            )
        };

        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    #[cfg(unix)]
    pub fn add_fd_full<F: AsRawFd>(
        &mut self,
        fd: F,
        priority: Priority,
        strategy: DuplicateStrategy,
    ) -> Result<(), error::UclError> {
        let file_fd = fd.as_raw_fd();
        let result = unsafe {
            ucl_parser_add_fd_full(
                self.parser,
                file_fd,
                priority.as_c_uint(),
                strategy,
                ucl_parse_type::UCL_PARSE_AUTO,
            )
        };

        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    /// Add the standard file variables to the `parser` based on the `filename` specified:
    ///
    /// - `$FILENAME`- a filename of ucl input
    /// - `$CURDIR` - a current directory of the input
    ///
    /// For example, if a filename param is `../something.conf` then the variables will have the following values:
    ///
    /// - `$FILENAME` - `../something.conf`
    /// - `$CURDIR` - `..`
    ///
    /// if need_expand parameter is true then all relative paths are expanded using realpath call. In this example if .. is /etc/dir then variables will have these values:
    ///
    /// - `$FILENAME` - `/etc/something.conf`
    /// - `$CURDIR` - `/etc`
    pub fn set_filevars<F: AsRef<Path>>(
        &mut self,
        filename: F,
        need_expand: bool,
    ) -> Result<(), error::UclError> {
        let file_path = utils::to_c_string(filename.as_ref().to_string_lossy());
        let result =
            unsafe { ucl_parser_set_filevars(self.parser, file_path.as_ptr(), need_expand) };
        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    /// Get a top object for a parser.
    ///
    /// Panics before acquiring a reference if the native reference count is exhausted.
    pub fn get_object(&mut self) -> Result<Object, error::UclError> {
        // The native getter decides whether the parser has a usable top object:
        // it tests the parser's internal error state, which the error code does
        // not mirror in either direction. But it also increments the count
        // without checking, so make sure there is room first.
        //
        // The first call needs no check: no object has escaped this parser yet.
        // That call's object is kept, because libucl 0.9.x retains the same top
        // object when more chunks are added, and it gives later calls a handle
        // to check. A checked acquisition that is released at once panics if
        // the count is exhausted and otherwise changes nothing. Nothing else can
        // acquire a reference between that check and the native getter, because
        // objects cannot leave this thread.
        if let Some(top) = &self.object {
            drop(top.clone());
        }
        let result = unsafe { ucl_parser_get_object(self.parser) };
        let object = Object::from_c_ptr(result).ok_or_else(|| self.get_error())?;
        if self.object.is_none() {
            self.object = Some(object.clone());
        }
        Ok(object)
    }

    /// Register new variable `$var` that should be replaced by the parser to the `value` string.
    /// Variables need to be registered _before_ they are referenced.
    ///
    /// #### Panics
    /// This function panics if either `var` or `value` has `\0`.
    pub fn register_variable<K: AsRef<str>, V: AsRef<str>>(
        &mut self,
        var: K,
        value: V,
    ) -> &mut Self {
        let var = utils::to_c_string(var);
        let value = utils::to_c_string(value);
        unsafe {
            ucl_parser_register_variable(self.parser, var.as_ptr(), value.as_ptr());
        };
        self
    }

    /// Register function as an unknown variable handler. Parser can only have one handler.
    ///
    /// - *handler* - a function pointer
    /// - *ud* - an opaque pointer that will be passed to a handler
    ///
    /// # Safety
    ///
    /// Both object behind `ud` and function behind `handler` need to live at least as long as the parser.
    pub unsafe fn set_variables_handler_raw(
        &mut self,
        handler: ucl_variable_handler,
        ud: *mut std::ffi::c_void,
    ) -> &mut Self {
        ucl_parser_set_variables_handler(self.parser, handler, ud);
        self
    }

    /// A safe counterpart of [`Parser::set_variable_handler_raw`](#method.set_variables_handler_raw). Unlike unsafe version this one takes ownership of a handler and ensures it stays alive as long as parser does.
    ///
    /// ### Caveats
    ///
    /// Parser can have only bar handler. In order to have multiple, please use [`CompoundHandler`](../../variable_handlers/compound/struct.CompoundHandler.html) to join multiple handlers into one.
    pub fn set_variables_handler(&mut self, handler: Box<dyn VariableHandler>) -> &mut Self {
        let mut handler = handler;
        let (state, callback) = handler.get_fn_ptr_and_data();
        self.var_handler = Some(handler);
        unsafe {
            self.set_variables_handler_raw(callback, state);
        }
        self
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe { ucl_parser_free(self.parser) }
    }
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser")
            .field("flags", &self.flags.0)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::VariableHandler;
    use crate::{UclErrorType, DEFAULT_DUPLICATE_STRATEGY};
    use bitflags::_core::ptr::slice_from_raw_parts;

    #[test]
    fn repeated_get_object_survives_parser_and_sees_new_chunks() {
        let mut parser = Parser::default();
        parser
            .add_chunk_full(
                "key = 42;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let first = parser.get_object().unwrap();
        let second = parser.get_object().unwrap();
        assert_eq!(first.as_ptr(), second.as_ptr());
        parser
            .add_chunk_full(
                "other = 17",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let third = parser.get_object().unwrap();
        drop(parser);
        drop(first);
        drop(second);
        assert_eq!(third.lookup("key").unwrap().as_i64(), Some(42));
        assert_eq!(third.lookup("other").unwrap().as_i64(), Some(17));
    }

    #[test]
    fn cached_object_does_not_hide_parser_error() {
        let mut parser = Parser::default();
        parser
            .add_chunk_full(
                "key = 42;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let _object = parser.get_object().unwrap();
        assert!(parser
            .add_chunk_full("invalid =", Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .is_err());
        assert!(parser.get_object().is_err());
    }

    #[test]
    fn cached_object_does_not_hide_error_without_error_code() {
        let mut parser = Parser::default();
        parser
            .add_chunk_full(
                "key = 42;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let _object = parser.get_object().unwrap();
        // An unknown macro puts libucl in its error state with error code 0.
        assert!(parser
            .add_chunk_full(
                ".nosuchmacro \"x\"\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY
            )
            .is_err());
        assert!(parser.get_object().is_err());
    }

    #[test]
    fn unmatched_brace_does_not_poison_later_get_object() {
        let mut parser = Parser::default();
        parser
            .add_chunk_full(
                "key = 42;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        // libucl reports this with an error code but stays out of its error state.
        assert!(parser
            .add_chunk_full(
                "section {\n a = 1;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY
            )
            .is_err());
        parser
            .add_chunk_full(
                "later = 2;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let object = parser.get_object().unwrap();
        assert_eq!(object.lookup("key").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn get_object_panics_before_overflow() {
        use crate::raw::object::refcount_test_support::SaturatedRef;
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let mut parser = Parser::default();
        parser
            .add_chunk_full(
                "key = 42;\n",
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();
        let original = parser.get_object().unwrap();
        let saturated = unsafe { SaturatedRef::new(original.as_ptr()) };
        let result = catch_unwind(AssertUnwindSafe(|| parser.get_object()));
        drop(saturated);
        assert!(result.is_err());
        drop(original);
        assert_eq!(
            parser.get_object().unwrap().lookup("key").unwrap().as_i64(),
            Some(42)
        );
    }

    #[test]
    fn incomplete_input() {
        let input = "key =";
        let mut parser = Parser::default();
        let chunk = parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY);
        assert!(chunk.is_err());
        let err = chunk.unwrap_err();
        assert_eq!(UclErrorType::Syntax, err.kind())
    }

    #[test]
    fn basic_vars_handler() {
        extern "C" fn simple(
            data: *const ::std::os::raw::c_uchar,
            len: usize,
            replace: *mut *mut ::std::os::raw::c_uchar,
            replace_len: *mut usize,
            need_free: *mut bool,
            _ud: *mut ::std::os::raw::c_void,
        ) -> bool {
            // Raw handlers receive libucl's buffer verbatim; on libucl 0.9.x the
            // expansion pass includes the closing brace and trailing bytes, so
            // trim down to the variable name. The safe `VariableHandler` path
            // does this automatically.
            let name_len = unsafe { crate::traits::variable_name_len(data, len) };
            let var = unsafe {
                let slice = slice_from_raw_parts(data, name_len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };
            unsafe {
                *need_free = false;
            }
            if var.eq("WWW") {
                let test = "asd";
                let size = test.len();
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
        }

        let input = r#"
        key = "${WWW}"
        "#;
        let mut parser = Parser::default();
        unsafe {
            parser.set_variables_handler_raw(Some(simple), std::ptr::null_mut());
        }
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let root = parser.get_object().unwrap();

        let looked_up_object = root.lookup("key").unwrap();
        dbg!(&looked_up_object);

        let object = looked_up_object.as_string().unwrap();
        assert_eq!("asd", object.as_str());
    }

    #[test]
    fn var_handler_with_closure() {
        let mut basic = |data: *const ::std::os::raw::c_uchar,
                         len: usize,
                         replace: *mut *mut ::std::os::raw::c_uchar,
                         replace_len: *mut usize,
                         need_free: *mut bool| {
            let var = unsafe {
                let slice = slice_from_raw_parts(data, len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };
            unsafe {
                *need_free = false;
            }
            if var.eq("WWW") {
                let test = "asd";
                let size = test.len();
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

        let (state, callback) = basic.get_fn_ptr_and_data();

        let input = r#"
        key = "${WWW}"
        "#;
        let mut parser = Parser::default();
        unsafe {
            parser.set_variables_handler_raw(callback, state);
        }
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let root = parser.get_object().unwrap();

        let looked_up_object = root.lookup("key").unwrap();
        dbg!(&looked_up_object);

        let object = looked_up_object.as_string().unwrap();
        assert_eq!("asd", object.as_str());
    }

    #[test]
    fn var_handler_safe() {
        let basic = |data: *const ::std::os::raw::c_uchar,
                     len: usize,
                     replace: *mut *mut ::std::os::raw::c_uchar,
                     replace_len: *mut usize,
                     need_free: *mut bool| {
            let var = unsafe {
                let slice = slice_from_raw_parts(data, len).as_ref().unwrap();
                std::str::from_utf8(slice).unwrap()
            };
            unsafe {
                *need_free = false;
            }
            if var.eq("WWW") {
                let test = "asd";
                let size = test.len();
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
        let input = r#"
        key = "${WWW}"
        "#;
        let mut parser = Parser::default();
        parser.set_variables_handler(Box::new(basic));
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let root = parser.get_object().unwrap();

        let looked_up_object = root.lookup("key").unwrap();
        dbg!(&looked_up_object);

        let object = looked_up_object.as_string().unwrap();
        assert_eq!("asd", object.as_str());
    }
}
