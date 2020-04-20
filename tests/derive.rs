use std::ptr::slice_from_raw_parts;
use uclicious::Uclicious;
use uclicious::{variable_handlers, Priority, UclError, DEFAULT_DUPLICATE_STRATEGY};

#[test]
fn derive_with_hook() {
    fn add_handlers(parser: &mut uclicious::Parser) -> Result<(), UclError> {
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

        let mut compound_handler = variable_handlers::compound::CompoundHandler::default();
        compound_handler.register_handler(Box::new(www));
        compound_handler.register_handler(Box::new(zzz));
        parser.set_variables_handler(Box::new(compound_handler));
        Ok(())
    }

    #[derive(Uclicious, Debug)]
    #[ucl(pre_source_hook = "add_handlers")]
    struct Test {
        key_one: String,
        key_two: String,
    }

    let input = r#"
        key_one = "${ZZZ}"
        key_two = "${WWW}"
        "#;

    let mut parser = Test::builder().unwrap();
    parser
        .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();

    let test = parser.build().unwrap();

    assert_eq!("dsa", test.key_one);
    assert_eq!("asd", test.key_two);
}
