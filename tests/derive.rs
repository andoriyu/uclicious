use std::net::{Ipv4Addr, SocketAddrV4};
use std::ptr::slice_from_raw_parts;
use uclicious::Uclicious;
use uclicious::{variable_handlers, Priority, UclError, DEFAULT_DUPLICATE_STRATEGY};

#[test]
fn derive_with_generics() {
    use uclicious::{FromObject, ObjectRef};

    #[derive(Uclicious, Debug)]
    struct Config<T = i64>
    where
        T: FromObject<ObjectRef>,
    {
        value: T,
    }

    let mut builder: ConfigBuilder = ConfigBuilder::new().unwrap();
    builder
        .add_chunk_full(
            "value = 42",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    assert_eq!(builder.build().unwrap().value, 42);

    let mut builder = Config::<String>::builder().unwrap();
    builder
        .add_chunk_full(
            "value = hello",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    assert_eq!(builder.build().unwrap().value, "hello");

    #[derive(Uclicious, Debug)]
    #[ucl(skip_builder)]
    struct Inner<T: FromObject<ObjectRef>> {
        value: T,
    }

    let mut parser = uclicious::Parser::default();
    parser
        .add_chunk_full(
            "value = 7; nested { value = 7; }",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    let root = parser.get_object().unwrap();
    let borrowed = <Inner<i64> as FromObject<&ObjectRef>>::try_from(&root).unwrap();
    let reference =
        <Inner<i64> as FromObject<ObjectRef>>::try_from(root.lookup("nested").unwrap()).unwrap();
    let owned = <Inner<i64> as FromObject<uclicious::Object>>::try_from(root).unwrap();
    assert_eq!((borrowed.value, reference.value, owned.value), (7, 7, 7));
}

#[test]
fn derive_with_lifetime_and_const_generics() {
    use std::marker::PhantomData;
    use uclicious::{ObjectError, ObjectRef};

    fn marker<T>(_: ObjectRef) -> Result<PhantomData<T>, ObjectError> {
        Ok(PhantomData)
    }

    #[derive(Uclicious)]
    struct Config<'a, const N: usize = 4> {
        value: i64,
        #[ucl(default, map = "marker")]
        marker: PhantomData<&'a [u8; N]>,
    }

    let mut builder: ConfigBuilder<'_> = Config::builder().unwrap();
    builder
        .add_chunk_full("value = 9", Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();
    let config = builder.build().unwrap();
    assert_eq!(config.value, 9);
    assert_eq!(config.marker, PhantomData);
}

#[test]
fn build_errors_support_context_and_preserve_sources() {
    use anyhow::Context;
    use std::error::Error;
    use uclicious::{BuildError, ObjectError};

    fn assert_thread_safe_error<E: Error + Send + Sync + 'static>() {}
    assert_thread_safe_error::<BuildError>();

    #[derive(Uclicious, Debug)]
    struct Config {
        value: i64,
    }

    let mut builder = Config::builder().unwrap();
    let parser_error = builder
        .add_chunk_full("value = {", Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap_err();
    let error = BuildError::from(parser_error);
    assert!(matches!(&error, BuildError::Parser(_)));
    assert!(error.source().unwrap().is::<UclError>());
    assert_eq!(error.to_string(), error.source().unwrap().to_string());

    let mut builder = Config::builder().unwrap();
    builder
        .add_chunk_full("other = 1", Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();
    let error = builder
        .build()
        .context("loading configuration")
        .unwrap_err();
    assert_eq!(error.to_string(), "loading configuration");
    assert!(matches!(
        error.downcast_ref::<BuildError>(),
        Some(BuildError::Object(ObjectError::KeyNotFound(key))) if key == "value"
    ));
    let source = error
        .downcast_ref::<BuildError>()
        .unwrap()
        .source()
        .unwrap();
    assert!(source.is::<ObjectError>());

    let mut builder = Config::builder().unwrap();
    builder
        .add_chunk_full(
            "value = 42",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    assert_eq!(
        builder
            .build()
            .context("loading configuration")
            .unwrap()
            .value,
        42
    );
}

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

#[test]
fn include_chunk() {
    #[derive(Uclicious, Debug)]
    #[ucl(include(chunk = r#"key_one = "asd""#))]
    struct Test {
        key_one: String,
    }
    let test = Test::builder().unwrap().build().unwrap();
    assert_eq!("asd", test.key_one);
}

#[test]
fn from_str() {
    #[derive(Uclicious, Debug)]
    #[ucl(include(chunk = r#"key_one = "0.0.0.0:8080""#))]
    struct Test {
        #[ucl(from_str)]
        key_one: SocketAddrV4,
    }

    let socket = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080);

    let test = Test::builder().unwrap().build().unwrap();
    assert_eq!(socket, test.key_one);
}

#[test]
fn include_chunk_with_macro() {
    #[derive(Uclicious, Debug)]
    #[ucl(include(chunk_static = "fixtures/key_one.ucl"))]
    struct Test {
        key_one: String,
    }
    let test = Test::builder().unwrap().build().unwrap();
    assert_eq!("asd", test.key_one);
}

#[test]
fn deny_unknown_fields() {
    #[derive(Uclicious, Debug)]
    #[ucl(deny_unknown_fields)]
    struct Strict {
        known: String,
    }

    // Known-only input builds fine.
    let mut builder = Strict::builder().unwrap();
    builder
        .add_chunk_full(
            r#"known = "ok""#,
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    assert_eq!("ok", builder.build().unwrap().known);

    // An extra key is rejected with a descriptive error.
    let mut builder = Strict::builder().unwrap();
    builder
        .add_chunk_full(
            "known = \"ok\"\nsurprise = \"boom\"",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    let err = builder.build().unwrap_err();
    let rendered = format!("{err}");
    assert!(
        rendered.contains("unknown field `surprise`"),
        "unexpected error: {}",
        rendered
    );
}

#[test]
fn allow_unknown_fields_by_default() {
    // Without `deny_unknown_fields`, extra keys are ignored (libucl default).
    #[derive(Uclicious, Debug)]
    struct Lax {
        known: String,
    }

    let mut builder = Lax::builder().unwrap();
    builder
        .add_chunk_full(
            "known = \"ok\"\nsurprise = \"boom\"",
            Priority::default(),
            DEFAULT_DUPLICATE_STRATEGY,
        )
        .unwrap();
    assert_eq!("ok", builder.build().unwrap().known);
}
