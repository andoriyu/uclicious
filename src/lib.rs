//! Universal configuration library parser with a lot of sugar.
//! Uclicious is built on top of [libucl](https://github.com/vstakhov/libucl) and exports safe, but raw API to that library:
//! ```rust
//!
//! ```

pub mod raw;
pub mod error;
pub mod from_object;

pub use error::{UclError, UclErrorType};
pub use raw::{DEFAULT_DUPLICATE_STRATEGY,DEFAULT_PARSER_FLAG,DuplicateStrategy,Priority,ParserFlags,Parser,ObjectError,Object,ObjectRef};
pub use from_object::FromObject;


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
        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
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
        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
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
        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
        let root = parser.get_object().unwrap();

        let expected = vec![1,2,3,4];
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
        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
        let root = parser.get_object().unwrap();

        let mut expected = HashMap::new();
        expected.insert(String::from("key"), String::from("value"));
        let actual: HashMap<String, String> = FromObject::try_from(root.lookup("dict").unwrap()).unwrap();
        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod derive_test {
    use super::*;
    use uclicious_derive::Uclicious;
    use std::path::PathBuf;
    use std::net::SocketAddr;
    use std::collections::HashMap;

    #[derive(Debug,Uclicious)]
    struct Connection {
        #[ucl(default)]
        enabled: bool,
        host: String,
        #[ucl(default = "420")]
        port: i64,
        buffer: u64,
        #[ucl(path = "type")]
        kind: String,
        locations: Vec<PathBuf>,
        addr: SocketAddr,
        extra: Extra,
        #[ucl(path = "subsection.host")]
        hosts: Vec<String>,
        #[ucl(default)]
        option: Option<String>,
        gates: HashMap<String, bool>,
    }

    #[derive(Debug, Uclicious)]
    #[ucl(skip_builder)]
    struct Extra {
        enabled: bool
    }
    #[test]
    fn showcase() {
        let mut builder = Connection::builder();

        let input = r#"
    enabled = yes
    host = "some.fake.url"
    buffer = 1mb
    type = "working"
    locations = "/etc/"
    addr = "127.0.0.1:80"
    extra = {
        enabled = on
    }
     subsection {
        host = [host1, host2]
    }
    gates {
        feature_1 = on
        feature_2 = off
        feature_3 = on
    }
    "#;

        builder.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
        let connection: Connection = builder.build().unwrap();
    }
}