use uclicious_derive::Uclicious;
use uclicious::*;
use std::path::PathBuf;
use std::net::SocketAddr;

#[derive(Debug,Uclicious)]
pub struct Connection {
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
    option: Option<String>,
}

#[derive(Debug, Uclicious)]
#[ucl(skip_builder)]
pub struct Extra {
    enabled: bool
}

fn main() {
    println!("Hello, world!");
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
    "#;

    builder.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
    let connection: Connection = builder.build().unwrap();
    dbg!(connection);
}