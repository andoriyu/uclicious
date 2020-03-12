pub mod raw;
pub mod error;
pub mod from_object;

pub use error::{UclError, UclErrorType};
pub use raw::{DEFAULT_DUPLICATE_STRATEGY,DEFAULT_PARSER_FLAG,DuplicateStrategy,Priority,ParserFlags,Parser,ObjectError,Object,ObjectRef};
pub use from_object::FromObject;


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