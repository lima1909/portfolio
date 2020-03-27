mod dotenv;
mod logging;

use dotenv::Dotenv;

fn main() {
    logging::init();

    Dotenv::new();
}
