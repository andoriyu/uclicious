use uclicious_derive::Uclicious;

#[derive(Debug,Uclicious)]
pub struct Connection {
    host: String,
    port: u32,
}
fn main() {
    println!("Hello, world!");
}
