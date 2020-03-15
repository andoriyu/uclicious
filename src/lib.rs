//! # Uclicious [![Build Status](https://dev.azure.com/andoriyu/personal/_apis/build/status/uclicious?branchName=master)](https://dev.azure.com/andoriyu/personal/_build/latest?definitionId=7&branchName=master) [![codecov](https://codecov.io/gh/andoriyu/uclicious/branch/master/graph/badge.svg)](https://codecov.io/gh/andoriyu/uclicious) [![docs.rs](https://docs.rs/uclicious/badge.svg)](https://docs.rs/uclicious) [![Crates.io](https://img.shields.io/crates/v/uclicious.svg)](https://crates.io/crates/uclicious)
//!
//! #### Uclicious is a wrapper around Universal Configuration Library (UCL) parser with a lot of sugar.
//!
//! Uclicious is built on top of [libucl](https://github.com/vstakhov/libucl).
//! It is much more complex than json or TOML, so I recommend reading documentaiton about it.
//! Library provides safe, but raw API to that library:
//! ```rust
//! use uclicious::*;
//! let mut parser = Parser::default();
//! let input = r#"
//! test_string = "no scope"
//! a_float = 3.14
//! an_integer = 69420
//! is_it_good = yes
//! buffer_size = 1KB
//! "#;
//! parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
//! let result = parser.get_object().unwrap();
//!
//! let lookup_result = result.lookup("test_string").unwrap().as_string().unwrap();
//! assert_eq!(lookup_result.as_str(), "no scope");
//!
//! let lookup_result = result.lookup("a_float").unwrap().as_f64().unwrap();
//! assert_eq!(lookup_result, 3.14f64);
//!
//! let lookup_result = result.lookup("an_integer").unwrap().as_i64().unwrap();
//! assert_eq!(lookup_result, 69420i64);
//!
//! let lookup_result = result.lookup("is_it_good").unwrap().as_bool().unwrap();
//! assert_eq!(lookup_result, true);
//!
//! let lookup_result = result.lookup("buffer_size").unwrap().as_i64().unwrap();
//! assert_eq!(lookup_result, 1024);
//! ```
//!
//! In order to get around rust rules library implemets its own trait FromObject for some basic types:
//! ```rust
//! use uclicious::*;
//! let mut parser = Parser::default();
//! let input = r#"
//! test_string = "no scope"
//! a_float = 3.14
//! an_integer = 69420
//! is_it_good = yes
//! buffer_size = 1KB
//! "#;
//! parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
//! let result = parser.get_object().unwrap();
//!
//! let lookup_result = result.lookup("is_it_good").unwrap();
//! let maybe: Option<bool> = FromObject::try_from(lookup_result).unwrap();
//! assert_eq!(Some(true), maybe);
//! ```
//! ### Automatic Derive
//!
//! On top of "raw" interface to libUCL, Uclicious provides an easy way to derive constructor for strucs:
//! ```rust
//! use uclicious::*;
//! use std::path::PathBuf;
//! use std::net::SocketAddr;
//! use std::collections::HashMap;
//!
//! #[derive(Debug,Uclicious)]
//! #[ucl(var(name = "test", value = "works"))]
//! struct Connection {
//!    #[ucl(default)]
//!    enabled: bool,
//!    host: String,
//!    #[ucl(default = "420")]
//!    port: i64,
//!    buffer: u64,
//!    #[ucl(path = "type")]
//!    kind: String,
//!    locations: Vec<PathBuf>,
//!    addr: SocketAddr,
//!    extra: Extra,
//!    #[ucl(path = "subsection.host")]
//!    hosts: Vec<String>,
//!    #[ucl(default)]
//!    option: Option<String>,
//!    gates: HashMap<String, bool>,
//! }
//!
//! #[derive(Debug,Uclicious)]
//! #[ucl(skip_builder)]
//! struct Extra {
//!    enabled: bool
//! }
//! let mut builder = Connection::builder().unwrap();
//!
//! let input = r#"
//!     enabled = yes
//!     host = "some.fake.url"
//!     buffer = 1mb
//!     type = $test
//!     locations = "/etc/"
//!     addr = "127.0.0.1:80"
//!     extra = {
//!        enabled = on
//!    }
//!     subsection {
//!        host = [host1, host2]
//!    }
//!    gates {
//!         feature_1 = on
//!         feature_2 = off
//!         feature_3 = on
//!    }"#;
//!
//! builder.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
//! let connection: Connection = builder.build().unwrap();
//! ```
//!
//! If you choose to derive builder then `::builder()` method will be added to target struct.
//!
//! ### Supported attributes (`#[ucl(..)]`)
//!
//! #### Structure level
//!
//!  - `skip_builder`
//!     - if set, then builder and builder methods won't be generated.
//!  - `parser(..)`
//!     - Optional attribute to configure inner parser.
//!     - Has following nested attributes:
//!         - `flags`
//!             - a path to function that returns flags.
//!  - `var(..)`
//!     - Optional attribute to register string variables with the parser.
//!     - Has following nested attributes:
//!         - `name`
//!             - A name of the variable without `$` part.
//!         - `value`
//!             - A string values for the variable.
//!             - Onlt string variables are supported by libUCL.
//!  - `include(..)`
//!     - Used to add files into the parser.
//!     - If file doesn't exist or failed to parse, then error will be returned in a constructor.
//!     - Has following nested attirbutes:
//!         - (required) `path(string)`
//!             - File path. Can be absolute or relative to CWD.
//!         - (optional) `priority(u32)`
//!             - 0-15 priority for the source. Consult the libUCL documentation for more information.
//!         - (optional) `strategy(uclicious::DuplicateStrategy)`
//!             - Strategy to use for duplicate keys. Consult the libUCL documentation for more information.
//!
//! #### Field level
//!
//!  - `default`
//!     - Use Default::default if key not found in object.
//!  - `default(expression)`
//!     - Use this _expression_ as value if key not found.
//!     - Could be a value or a function call.
//!  - `path(string)`
//!     - By default field name is used as path.
//!     - If set that would be used as a key.
//!     - dot notation for key is supported.
//!
//! ### Additional notes
//!  - If target type is an array, but key is a single value — an implicit list is created.
//!  - Automatic derive on enums is not supported, but you can implement it yourself.
//!  - I have a few more features I want to implement before publishing this crate:
//!     - Ability to add variables.
//!     - Ability to add macross handlers.
//!     - (maybe) configure parser that us used for derived builder with atrributes.
//!     - (done) add sources to parser with attributes.
//!
//! ## Contributing
//!
//! PRs, feature requests, bug reports are welcome. I won't be adding CoC  — be civilized.
//!
//! ## Goals
//!  - Provider safe and convient configuration library
//!  - Automatic derive, so you don't have to think about parser object
//!
//! ### Not Goals
//!  - Providing UCL Object generation tools is not a goal for this project
//!  - 1:1 interface to libUCL
//!  - sugar inside `raw` module
//!
//! ## Special thanks
//!  - [draft6](https://github.com/draft6) and [hauleth](https://github.com/hauleth)
//!     - libucl-rs was a good starting point
//!     - Type wrappers pretty much copied from there
//!  - [colin-kiegel](https://github.com/colin-kiegel)
//!     - Rust-derive-builder was used as a starting point for uclicious-derive
//!     - Very well documented proc_macro crate, do recommend
//!
//! ## LICENSE
//!
//! [BSD-2-Clause](https://github.com/andoriyu/uclicious/blob/master/LICENSE).
pub mod error;
pub mod raw;
pub mod traits;

pub use error::{UclError, UclErrorType};
pub use raw::{
    DuplicateStrategy, Object, ObjectError, ObjectRef, Parser, ParserFlags, Priority,
    DEFAULT_DUPLICATE_STRATEGY, DEFAULT_PARSER_FLAG,
};
pub use traits::FromObject;

#[cfg(feature = "uclicious_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate uclicious_derive;

#[cfg(feature = "uclicious_derive")]
#[doc(hidden)]
pub use uclicious_derive::*;

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn primitives_from_object() {
        let input = r#"
            i64 = 1
            i32 = 1
            i16 = 1
            i8  = 1
            u64 = 1
            u32 = 1
            u16 = 1
            u8  = 1
            f64 = 3.14
            bool = true
        "#;

        let mut parser = Parser::default();
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let root = parser.get_object().unwrap();

        let boolean: bool = FromObject::try_from(root.lookup("bool").unwrap()).unwrap();
        assert_eq!(true, boolean);

        let float64: f64 = FromObject::try_from(root.lookup("f64").unwrap()).unwrap();
        assert_eq!(3.14, float64);

        let int64: i64 = FromObject::try_from(root.lookup("i64").unwrap()).unwrap();
        assert_eq!(1, int64);

        let int32: i32 = FromObject::try_from(root.lookup("i32").unwrap()).unwrap();
        assert_eq!(1, int32);

        let int16: i16 = FromObject::try_from(root.lookup("i16").unwrap()).unwrap();
        assert_eq!(1, int16);

        let int8: i8 = FromObject::try_from(root.lookup("i8").unwrap()).unwrap();
        assert_eq!(1, int8);

        let uint64: u64 = FromObject::try_from(root.lookup("u64").unwrap()).unwrap();
        assert_eq!(1, uint64);

        let uint32: u32 = FromObject::try_from(root.lookup("u32").unwrap()).unwrap();
        assert_eq!(1, uint32);

        let uint16: u16 = FromObject::try_from(root.lookup("u16").unwrap()).unwrap();
        assert_eq!(1, uint16);

        let uint8: u8 = FromObject::try_from(root.lookup("u8").unwrap()).unwrap();
        assert_eq!(1, uint8);
    }

    #[test]
    fn string_from_object() {
        let input = r#"
            test_string = "a string"
        "#;

        let mut parser = Parser::default();
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let root = parser.get_object().unwrap();

        let expected = String::from("a string");
        let actual: String = FromObject::try_from(root.lookup("test_string").unwrap()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn array_from_object() {
        let input = r#"
            list = [1,2,3,4]
            implicit_list = 1
        "#;

        let mut parser = Parser::default();
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let root = parser.get_object().unwrap();

        let expected = vec![1, 2, 3, 4];
        let actual: Vec<i32> = FromObject::try_from(root.lookup("list").unwrap()).unwrap();
        assert_eq!(expected, actual);

        let expected = vec![1];
        let actual: Vec<i32> = FromObject::try_from(root.lookup("implicit_list").unwrap()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn hashmap_from_object() {
        let input = r#"
            dict {
                key = value
            }
        "#;

        let mut parser = Parser::default();
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let root = parser.get_object().unwrap();

        let mut expected = HashMap::new();
        expected.insert(String::from("key"), String::from("value"));
        let actual: HashMap<String, String> =
            FromObject::try_from(root.lookup("dict").unwrap()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_var() {
        let input = r#"
            dst = $dst
        "#;

        let mut parser = Parser::default();
        parser
            .register_variable("dst", "/etc/")
            .register_variable("int", "1");
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let root = parser.get_object().unwrap();

        let expected = "/etc/".to_string();
        let actual = root.lookup("dst").unwrap().as_string().unwrap();
        assert_eq!(expected, actual);
    }
}
