use std::env;

use cryptograms::ciphers;

fn main() {
    let _ = env_logger::init();

    if env::args().any(|arg| arg == "print") {
        cryptograms::print_schema();
    } else {
        //cryptograms::make_server();
        println!("{}", ciphers::encrypt("", ciphers::Type::Cryptarithm, None));
    }
}
