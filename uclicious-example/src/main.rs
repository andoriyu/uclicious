use uclicious_derive::Uclicious;
use uclicious;

#[derive(Debug,Uclicious)]
pub struct Connection {
//    host: String,
    #[ucl(default = "420")]
    port: i64,
//    #[ucl(path = "type")]
//    kind: String,
}
fn main() {
    println!("Hello, world!");
}


mod wat {

}