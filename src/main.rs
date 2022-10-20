use std::env;

fn main() {
    let _ = env_logger::init();

    if env::args().any(|arg| arg == "print") {
        cryptograms::print_schema();
    } else {
        cryptograms::make_server();
    }
}
