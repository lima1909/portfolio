mod authentication;
mod dotenv;
mod logging;

use log::{debug, error, info};
use std::env;

fn main() {
    logging::init();

    match env::var("PRIVATE_KEY") {
        Ok(pk) => {
            debug!("KEY: {:?}", &pk[..50]);
            match authentication::generate_jwt(authentication::Claim::new(), &pk) {
                Ok(token) => {
                    info!("Token: {}", &token[..10]);
                    let mut dotenv = dotenv::Dotenv::new();
                    dotenv.put(dotenv::KEY_JWT_TOKEN.to_string(), token);
                    if let Err(msg) = dotenv.write_to_file() {
                        error!("{}", msg);
                    }
                }
                Err(msg) => error!("{}", msg),
            };
        }
        Err(msg) => error!("Err by read env PRIVATE_KEY: {}", msg),
    };
}
