use std::env;

fn main() {
    env_logger::init();

    if env::args().any(|arg| arg == "print") {
        cryptograms::print_schema();
    } else {
        log::info!("Starting server.");
        cryptograms::make_server();
    }
}
