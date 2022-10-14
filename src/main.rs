use std::env;

fn main() {
    if env::args().any(|arg| arg == "print") {
        cryptograms::print_schema();
    } else {
        cryptograms::make_server();
    }
}
