mod authentication;
mod dotenv;
mod logging;

mod gcloud;
use gcloud::auth::{Auth, JwtToken};
use gcloud::datastore::Datastore;

use log::{debug, error};
use std::env;

fn main() {
    logging::init();

    match env::var("PRIVATE_KEY") {
        Ok(pk) => {
            debug!("KEY: {:?}", &pk[..50]);
            match JwtToken::create(&pk) {
                Ok(auth) => {
                    // write to dot-env-file
                    // temporary solution
                    let mut dotenv = dotenv::Dotenv::new();
                    dotenv.put(
                        dotenv::KEY_JWT_TOKEN.to_string(),
                        auth.jwt_token.to_string(),
                    );
                    if let Err(msg) = dotenv.write_to_file() {
                        error!("{}", msg);
                    }

                    // do a lookup to the datastore
                    let s = Datastore::new("goheros-207118", &auth);
                    s.lookup("heroes", "Protocol", 4851027920551936);
                }
                Err(msg) => error!("{}", msg),
            }
        }
        Err(msg) => error!("Err by read env PRIVATE_KEY: {}", msg),
    };
}
