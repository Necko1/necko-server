use necko_core::hello_world as core_hello;
use necko_protocol::hello_world as protocol_hello;

fn main() {
    println!("{}", core_hello());
    println!("{}", protocol_hello());
}
