use uclicious::raw::*;

fn main() {
    let mut parser = Parser::default();
    let input = r#"game = on"#;

    parser
        .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();

    let result = parser.get_object().unwrap();
    let lookup_result = result.lookup("game").unwrap();
    assert!(lookup_result.as_bool().unwrap());
    let _ = lookup_result;

    let lookup_result = result.lookup("game").unwrap();
    assert!(lookup_result.as_bool().unwrap());
}
