pub mod raw;
pub mod error;

pub use error::{UclError, UclErrorType};
pub use raw::{DEFAULT_DUPLICATE_STRATEGY,DEFAULT_PARSER_FLAG,DuplicateStrategy,Priority,ParserFlags,Parser,ObjectError,Object,ObjectRef};

/// A new type get around rust rules... just as if you were using
pub struct Optional