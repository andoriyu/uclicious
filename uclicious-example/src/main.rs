use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::path::PathBuf;
use uclicious::*;

#[derive(Debug, Eq, PartialEq)]
pub struct WrappedInt(pub i64);

impl From<i64> for WrappedInt {
    fn from(src: i64) -> Self {
        WrappedInt(src)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Visibility {
    Hidden,
    Visible,
}

impl TryFrom<String> for Visibility {
    type Error = ObjectError;

    fn try_from(src: String) -> Result<Self, Self::Error> {
        match src.to_lowercase().as_str() {
            "hidden" => Ok(Visibility::Hidden),
            "visible" => Ok(Visibility::Visible),
            _ => Err(ObjectError::other(format!(
                "{} is not supported visibility type",
                src
            ))),
        }
    }
}
#[derive(Debug, Uclicious)]
#[ucl(var(name = "test", value = "works"))]
#[ucl(include(path = "test.ucl"))]
#[ucl(include(
    path = "another-test.ucl",
    strategy = "DuplicateStrategy::UCL_DUPLICATE_MERGE",
    priority = 10
))]
pub struct Connection {
    #[ucl(default)]
    enabled: bool,
    host: String,
    #[ucl(default = "420", validate = "validators::is_positive")]
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
    #[ucl(from = "i64")]
    wrapped_int: WrappedInt,
    #[ucl(try_from = "String", validate = "validators::is_okay")]
    visibility: Visibility,
}

#[derive(Debug, Uclicious)]
#[ucl(skip_builder)]
pub struct Extra {
    enabled: bool,
}
mod validators {
    use crate::Visibility;
    use uclicious::ObjectError;

    #[inline(always)]
    pub fn is_positive(path: &str, val: &i64) -> Result<(), ObjectError> {
        if *val > 0 {
            Ok(())
        } else {
            Err(ObjectError::other(format!(
                "{} is not a positive number",
                path
            )))
        }
    }

    pub fn is_okay(_path: &str, _val: &Visibility) -> Result<(), ObjectError> {
        Ok(())
    }
}
fn main() {
    let mut builder = Connection::builder().unwrap();

    let input = r#"
    enabled = yes
    host = "some.fake.url"
    port = 20
    buffer = 1mb
    type = $test
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
    wrapped_int = 1
    visibility = "hidden"
    "#;

    builder
        .add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();
    let connection: Connection = builder.build().unwrap();
    dbg!(connection);
}
