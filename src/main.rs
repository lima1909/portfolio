mod dotenv;

use dotenv::Dotenv;

fn main() {
    Dotenv::new().load();
}
