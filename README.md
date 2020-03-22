<!-- cargo-sync-readme start -->

# Uclicious [![Build Status](https://dev.azure.com/andoriyu/personal/_apis/build/status/andoriyu.uclicious?branchName=master)](https://dev.azure.com/andoriyu/personal/_build/latest?definitionId=7&branchName=master) [![codecov](https://codecov.io/gh/andoriyu/uclicious/branch/master/graph/badge.svg)](https://codecov.io/gh/andoriyu/uclicious) [![docs.rs](https://docs.rs/uclicious/badge.svg)](https://docs.rs/uclicious) [![Crates.io](https://img.shields.io/crates/v/uclicious.svg)](https://crates.io/crates/uclicious)

#### Uclicious is a wrapper around Universal Configuration Library (UCL) parser with a lot of sugar.

Uclicious is built on top of [libucl](https://github.com/vstakhov/libucl).
It is much more complex than json or TOML, so I recommend reading documentaiton about it.
Library provides safe, but raw API to that library:
```rust
use uclicious::*;
let mut parser = Parser::default();
let input = r#"
test_string = "no scope"
a_float = 3.14
an_integer = 69420
is_it_good = yes
buffer_size = 1KB
interval = 1s
"#;
parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
let result = parser.get_object().unwrap();

let lookup_result = result.lookup("test_string").unwrap().as_string().unwrap();
assert_eq!(lookup_result.as_str(), "no scope");

let lookup_result = result.lookup("a_float").unwrap().as_f64().unwrap();
assert_eq!(lookup_result, 3.14f64);

let lookup_result = result.lookup("an_integer").unwrap().as_i64().unwrap();
assert_eq!(lookup_result, 69420i64);

let lookup_result = result.lookup("is_it_good").unwrap().as_bool().unwrap();
assert_eq!(lookup_result, true);

let lookup_result = result.lookup("buffer_size").unwrap().as_i64().unwrap();
assert_eq!(lookup_result, 1024);
let lookup_result = result.lookup("interval").unwrap().as_time().unwrap();
assert_eq!(lookup_result, 1.0f64);
```

In order to get around rust rules library implemets its own trait FromObject for some basic types:
```rust
use uclicious::*;
let mut parser = Parser::default();
let input = r#"
test_string = "no scope"
a_float = 3.14
an_integer = 69420
is_it_good = yes
buffer_size = 1KB
"#;
parser.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
let result = parser.get_object().unwrap();

let lookup_result = result.lookup("is_it_good").unwrap();
let maybe: Option<bool> = FromObject::try_from(lookup_result).unwrap();
assert_eq!(Some(true), maybe);
```
### Automatic Derive

On top of "raw" interface to libUCL, Uclicious provides an easy way to derive constructor for strucs:
```rust
use uclicious::*;
use std::path::PathBuf;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug,Uclicious)]
#[ucl(var(name = "test", value = "works"))]
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
   interval: Duration,
}

#[derive(Debug,Uclicious)]
#[ucl(skip_builder)]
struct Extra {
   enabled: bool
}
let mut builder = Connection::builder().unwrap();

let input = r#"
    enabled = yes
    host = "some.fake.url"
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
   interval = 10ms
   gates {
        feature_1 = on
        feature_2 = off
        feature_3 = on
   }"#;

builder.add_chunk_full(input, Priority::default(), DEFAULT_DUPLICATE_STRATEGY).unwrap();
let connection: Connection = builder.build().unwrap();
```

If you choose to derive builder then `::builder()` method will be added to target struct.

### Supported attributes (`#[ucl(..)]`)

#### Structure level

 - `skip_builder`
    - if set, then builder and builder methods won't be generated.
 - `parser(..)`
    - Optional attribute to configure inner parser.
    - Has following nested attributes:
        - `flags`
            - a path to function that returns flags.
        - `filevars(..)`
            - call `set_filevars` on a parser.
            - Has following nested attributes:
                - `path`
                    - a string representation of filepath.
                - `expand`
                    - (optional) if set, then variables would be expanded to absolute.
 - `var(..)`
    - Optional attribute to register string variables with the parser.
    - Has following nested attributes:
        - `name`
            - A name of the variable without `$` part.
        - `value`
            - A string values for the variable.
            - Onlt string variables are supported by libUCL.
 - `include(..)`
    - Used to add files into the parser.
    - If file doesn't exist or failed to parse, then error will be returned in a constructor.
    - Has following nested attirbutes:
        - (required) `path(string)`
            - File path. Can be absolute or relative to CWD.
        - (optional) `priority(u32)`
            - 0-15 priority for the source. Consult the libUCL documentation for more information.
        - (optional) `strategy(uclicious::DuplicateStrategy)`
            - Strategy to use for duplicate keys. Consult the libUCL documentation for more information.

#### Field level

 - `default`
    - Use Default::default if key not found in object.
 - `default(expression)`
    - Use this _expression_ as value if key not found.
    - Could be a value or a function call.
 - `path(string)`
    - By default field name is used as path.
    - If set that would be used as a key.
    - dot notation for key is supported.

### Additional notes
 - If target type is an array, but key is a single value — an implicit list is created.
 - Automatic derive on enums is not supported, but you can implement it yourself.
 - I have a few more features I want to implement before publishing this crate:
    - Ability to add variables.
    - Ability to add macross handlers.
    - (maybe) configure parser that us used for derived builder with atrributes.
    - (done) add sources to parser with attributes.

## Contributing

PRs, feature requests, bug reports are welcome. I won't be adding CoC  — be civilized.

#### Particular Contributions of Interest

 - Optimize derive code.
 - Improve documentation — I often write late and night and some it might look like a word soup.
 - Better tests
 - `from` and `try_from` like in serder [#3]
 - Glob support in derive parser section
 

## Goals
 - Provider safe and convient configuration library
 - Automatic derive, so you don't have to think about parser object

### Not Goals
 - Providing UCL Object generation tools is not a goal for this project
 - 1:1 interface to libUCL
 - sugar inside `raw` module

## Special thanks
 - [draft6](https://github.com/draft6) and [hauleth](https://github.com/hauleth)
    - libucl-rs was a good starting point
    - Type wrappers pretty much copied from there
 - [colin-kiegel](https://github.com/colin-kiegel)
    - Rust-derive-builder was used as a starting point for uclicious-derive
    - Very well documented proc_macro crate, do recommend

## LICENSE

[BSD-2-Clause](https://github.com/andoriyu/uclicious/blob/master/LICENSE).

<!-- cargo-sync-readme end -->
