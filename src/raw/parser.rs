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
use libucl_bind::{ucl_parser, ucl_parser_new, ucl_parser_get_error_code, ucl_parser_get_error, ucl_parser_add_chunk_full, ucl_parse_type, ucl_parser_add_file_full, ucl_parser_free, ucl_parser_get_object, ucl_parser_add_fd_full, ucl_parser_set_filevars};
use crate::raw::{Priority, DuplicateStrategy};

#[cfg(unix)] use std::os::unix::io::AsRawFd;

use crate::error;
use std::path::Path;
use super::{utils,ParserFlags,DEFAULT_PARSER_FLAG};
use crate::raw::object::{Object};
use std::fmt;

/// Raw parser object.
pub struct Parser {
    parser: *mut ucl_parser,
    flags: ParserFlags
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
            flags
        }
    }


    /// Add a chunk of text to the parser. String must:
    /// - not have `\0` character;
    /// - must be valid UCL object;
    pub fn add_chunk_full<C: AsRef<str>>(&mut self, chunk: C, priority: Priority, strategy: DuplicateStrategy) -> Result<(), error::UclError> {
        let chunk = chunk.as_ref();
        let result = unsafe {
            ucl_parser_add_chunk_full(self.parser, chunk.as_ptr(), chunk.as_bytes().len(), priority.as_c_uint(), strategy, ucl_parse_type::UCL_PARSE_AUTO)
        };
        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    /// Add a file by a file path to the parser. This function uses mmap call to load file, therefore, it should not be shrunk during parsing.
    pub fn add_file_full<F: AsRef<Path>>(&mut self, file: F, priority: Priority, strategy: DuplicateStrategy) -> Result<(), error::UclError> {
        let file_path = utils::to_c_string(file.as_ref().to_string_lossy());
        let result = unsafe {
            ucl_parser_add_file_full(self.parser, file_path.as_ptr(), priority.as_c_uint(), strategy, ucl_parse_type::UCL_PARSE_AUTO)
        };

        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    #[cfg(unix)]
    pub fn add_fd_full<F: AsRawFd>(&mut self, fd: F, priority: Priority, strategy: DuplicateStrategy) -> Result<(), error::UclError> {
        let file_fd = fd.as_raw_fd();
        let result = unsafe {
            ucl_parser_add_fd_full(self.parser, file_fd, priority.as_c_uint(), strategy, ucl_parse_type::UCL_PARSE_AUTO)
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
    pub fn set_filevars<F: AsRef<Path>>(&mut self, filename: F, need_expand: bool) -> Result<(), error::UclError> {
        let file_path = utils::to_c_string(filename.as_ref().to_string_lossy());
        let result = unsafe {
            ucl_parser_set_filevars(self.parser, file_path.as_ptr(), need_expand)
        };
        if result {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    /// Get a top object for a parser.
    pub fn get_object(&mut self) -> Result<Object, error::UclError> {
        let result = unsafe { ucl_parser_get_object(self.parser) };
        if !result.is_null() {
            Ok(Object::from_c_ptr(result).expect("Failed to build object from non-null pointer"))
        } else {
            Err(self.get_error())
        }
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