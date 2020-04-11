mod authentication;
mod dotenv;
mod logging;

mod gcloud;
use gcloud::auth::{Auth, JwtToken};
use gcloud::datastore::{Datastore, Error};

use log::error;
use serde::{Deserialize, Serialize};
use std::time::Instant;

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

    match JwtToken::from_env_private_key() {
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
            let q = auth.to_query_url();
            let s = Datastore::new("goheros-207118", &q);
            let now = Instant::now();
            let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 4851027920551936);
            println!("lookup result ({}ms): \n{:?}", now.elapsed().as_millis(), r);

            let now = Instant::now();
            let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 5066702320566272);
            println!("lookup result ({}ms): \n{:?}", now.elapsed().as_millis(), r);
        }
        Err(msg) => error!("{}", msg),
    };
}
