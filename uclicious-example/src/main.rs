use uclicious_derive::Uclicious;
use uclicious::*;

#[derive(Debug,Uclicious)]
pub struct Connection {
    #[ucl(default)]
    enabled: bool,
    host: String,
    #[ucl(default = "420")]
    port: i64,
    buffer: u64,
//    #[ucl(path = "type")]
//    kind: String,
}
fn main() {
    println!("Hello, world!");
    let mut builder = Connection::builder();

    let input = r#"
    enabled = yes
    host = "some.fake.url"
    buffer = 1mb
    "#;

    builder.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
    let connection: Connection = builder.build().unwrap();
    dbg!(connection);
}