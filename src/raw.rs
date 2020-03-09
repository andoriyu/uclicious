pub mod parser;
pub mod utils;
pub mod priority;
pub mod object;
pub mod iterator;

pub use priority::Priority;
pub use parser::Parser;

pub type DuplicateStrategy = libucl_bind::ucl_duplicate_strategy;
pub const DEFAULT_DUPLICATE_STRATEGY:DuplicateStrategy = DuplicateStrategy::UCL_DUPLICATE_APPEND;

pub type ParserFlags = libucl_bind::ucl_parser_flags;
pub const DEFAULT_PARSER_FLAG: ParserFlags = ParserFlags::UCL_PARSER_DEFAULT;


#[cfg(test)]
mod test {
    use super::*;
    use crate::raw::parser::Parser;
    use libucl_bind::ucl_type_t;
    use crate::raw::iterator::IterMut;
    use crate::raw::object::ObjectRef;

    #[test]
    fn string_parsing() {
        let mut parser = Parser::default();
        let input = r#"test_string = "test_string""#;

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("test_string").unwrap().as_string().unwrap();

        assert_eq!(lookup_result.as_str(), "test_string");
    }

    #[test]
    fn integer_parsing() {
        let mut parser = Parser::default();
        let input = r#"one_kb = 1kb"#;

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("one_kb").unwrap().as_i64().unwrap();

        assert_eq!(1024, lookup_result);
    }

    #[test]
    fn float_parsing() {
        let mut parser = Parser::default();
        let input = r#"pi = 3.14"#;

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("pi").unwrap().as_f64().unwrap();

        assert_eq!(3.14, lookup_result);
    }

    #[test]
    fn boolean_parsing() {
        let mut parser = Parser::default();
        let input = r#"game = on"#;

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("game").unwrap().as_bool().unwrap();

        assert_eq!(true, lookup_result);
    }

    #[test]
    fn into_iter_primitive() {
        let mut parser = Parser::default();
        let input = "features = [1,2,3]";
        parser.add_chunk_full(input, Priority::default(), DuplicateStrategy::UCL_DUPLICATE_MERGE).unwrap();
        let input = "features = [4,5,6]";
        parser.add_chunk_full(input, Priority::default(), DuplicateStrategy::UCL_DUPLICATE_MERGE).unwrap();

        let result = parser.get_object().unwrap();
        let array = result.lookup("features").unwrap();

        assert_eq!(ucl_type_t::UCL_ARRAY, array.kind());



        let actual: Vec<i64> = array.iter()
            .map(|obj| obj.as_i64().unwrap())
            .collect();

        let expected = vec![1,2,3,4,5,6i64];
        assert_eq!(expected, actual);

    }

    #[test]
    fn boolean_segfault() {
        let mut parser = Parser::default();
        let input = r#"game = on"#;

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

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

        parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();

        let result = parser.get_object().unwrap();
        let lookup_result = result.lookup("game").unwrap();
        assert_eq!(true, lookup_result.as_bool().unwrap());
        drop(lookup_result);

        let lookup_result = result.lookup("game").unwrap();
        assert_eq!(true, lookup_result.as_bool().unwrap());
    }
}