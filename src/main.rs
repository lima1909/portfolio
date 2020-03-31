mod authentication;
mod dotenv;
mod logging;

mod gcloud;

use log::{debug, error};
use std::env;

use gcloud::auth::Auth;
use gcloud::datastore::Datastore;
// use gcloud::Store;

fn main() {
    logging::init();

    match env::var("PRIVATE_KEY") {
        Ok(pk) => {
            debug!("KEY: {:?}", &pk[..50]);
            match Auth::create(Auth::JwtToken(""), pk) {
                Ok(auth) => {
                    let s = Datastore::new(String::from("goheros-207118"), auth);
                    s.lookup("heroes".to_owned(), "Protocol".to_owned(), 4851027920551936);
                }
                Err(msg) => error!("{}", msg),
            }
        }
        Err(msg) => error!("Err by read env PRIVATE_KEY: {}", msg),
    };
}
