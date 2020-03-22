//! Low level interface to libUCL.

pub mod iterator;
pub mod object;
pub mod parser;
pub mod priority;
mod utils;

pub use object::{Object, ObjectError, ObjectRef};
pub use parser::Parser;
pub use priority::Priority;

/// Strategy to use when sources have duplicate keys.
pub type DuplicateStrategy = libucl_bind::ucl_duplicate_strategy;
/// Default strategy is append. Consult libUCL docs for more information.
pub const DEFAULT_DUPLICATE_STRATEGY: DuplicateStrategy = DuplicateStrategy::UCL_DUPLICATE_APPEND;

/// Parser flags.
pub type ParserFlags = libucl_bind::ucl_parser_flags;
#[doc(hidden)]
pub const DEFAULT_PARSER_FLAG: ParserFlags = ParserFlags::UCL_PARSER_DEFAULT;

#[cfg(test)]
mod test {
    use super::*;
    use crate::raw::object::Object;
    use crate::raw::parser::Parser;
    use libucl_bind::ucl_type_t;
    use std::error::Error;
    use std::path::Display;

    #[test]
    fn string_parsing() {
        let mut parser = Parser::default();
        let input = r#"test_string = "test_string""#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("test_string").unwrap().as_string().unwrap();

        assert_eq!(lookup_result.as_str(), "test_string");
    }

    #[test]
    fn integer_parsing() {
        let mut parser = Parser::default();
        let input = r#"one_kb = 1kb"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("one_kb").unwrap().as_i64().unwrap();

        assert_eq!(1024, lookup_result);
    }

    #[test]
    fn float_parsing() {
        let mut parser = Parser::default();
        let input = r#"pi = 3.14"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("pi").unwrap().as_f64().unwrap();

        assert_eq!(3.14, lookup_result);
    }

    #[test]
    fn boolean_parsing() {
        let mut parser = Parser::default();
        let input = r#"game = on"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("game").unwrap().as_bool().unwrap();

        assert_eq!(true, lookup_result);
    }

    #[test]
    fn into_iter_primitive() {
        let mut parser = Parser::default();
        let input = "features = [1,2,3]";
        parser
            .add_chunk_full(
                input,
                Priority::default(),
                DuplicateStrategy::UCL_DUPLICATE_MERGE,
            )
            .unwrap();
        let input = "features = [4,5,6]";
        parser
            .add_chunk_full(
                input,
                Priority::default(),
                DuplicateStrategy::UCL_DUPLICATE_MERGE,
            )
            .unwrap();

        let result = parser.get_object().unwrap();
        let array = result.lookup("features").unwrap();

        assert_eq!(ucl_type_t::UCL_ARRAY, array.kind());

        let actual: Vec<i64> = array.iter().map(|obj| obj.as_i64().unwrap()).collect();

        let expected = vec![1, 2, 3, 4, 5, 6i64];
        assert_eq!(expected, actual);
    }

    #[test]
    fn boolean_segfault() {
        let mut parser = Parser::default();
        let input = r#"game = on"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("game").unwrap();

        drop(result);
        drop(parser);

        dbg!(lookup_result.as_bool().unwrap());
    }

    #[test]
    fn boolean_double_free() {
        let mut parser = Parser::default();
        let input = r#"game = on"#;

        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("game").unwrap();
        assert_eq!(true, lookup_result.as_bool().unwrap());
        drop(lookup_result);

        let lookup_result = result.lookup("game").unwrap();
        assert_eq!(true, lookup_result.as_bool().unwrap());
    }

    #[test]
    fn iter_object() {
        let mut parser = Parser::default();
        let input = "game = on\nfreedom = yes";
        parser
            .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();

        let result = parser.get_object().unwrap();
        assert_eq!(ucl_type_t::UCL_OBJECT, result.kind());

        assert_eq!(2, result.iter().count());

        for obj in result.iter() {
            assert_eq!(ucl_type_t::UCL_BOOLEAN, obj.kind());
        }
    }

    #[test]
    fn object_from_primitive() {
        let obj_boolean = Object::from(false);
        assert_eq!(ucl_type_t::UCL_BOOLEAN, obj_boolean.kind());
        assert_eq!(false, obj_boolean.as_bool().unwrap());

        let obj_i64 = Object::from(1776i64);
        assert_eq!(ucl_type_t::UCL_INT, obj_i64.kind());
        assert_eq!(1776, obj_i64.as_i64().unwrap());

        let obj_f64 = Object::from(3.14);
        assert_eq!(ucl_type_t::UCL_FLOAT, obj_f64.kind());
        assert_eq!(3.14, obj_f64.as_f64().unwrap());

        let obj_str = Object::from("a string without null");
        assert_eq!(ucl_type_t::UCL_STRING, obj_str.kind());
        assert_eq!("a string without null", obj_str.as_string().unwrap());
    }

    #[test]
    fn dyn_error() {
        let err = std::io::Error::from_raw_os_error(42);
        let err = ObjectError::other(err);

        assert!(err.to_string().contains("os error 42"));
    }
}
