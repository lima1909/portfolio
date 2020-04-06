mod authentication;
mod dotenv;
mod logging;

mod gcloud;
use gcloud::auth::{Auth, JwtToken};
use gcloud::datastore::{Datastore, ResponseError};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug)]
struct Hero {
    #[serde(rename(deserialize = "HeroID"))]
    hero_id: isize,
    #[serde(rename(deserialize = "Note"))]
    note: String,
    #[serde(rename(deserialize = "Action"))]
    action: String,
    #[serde(rename(deserialize = "Time"))]
    time: String,
}

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
                    let r: Result<Hero, ResponseError> =
                        s.lookup("heroes", "Protocol", 4851027920551936);
                    println!("lookup result: \n{:?}", r);
                }
                Err(msg) => error!("{}", msg),
            }
        }
        Err(msg) => error!("Err by read env PRIVATE_KEY: {}", msg),
    };
}
