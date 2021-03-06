mod authentication;
mod dotenv;
mod logging;

mod gcloud;
use gcloud::auth::{Auth, JwtToken};
use gcloud::datastore::query::{Filter, Operator, Value};
use gcloud::datastore::Datastore;
use gcloud::Error;

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

    match JwtToken::from_env_private_key(authentication::Claim::new()) {
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
            let q = auth.to_url_query();
            let s = Datastore::new("goheros-207118", &q);
            let now = Instant::now();
            let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 4851027920551936);
            println!("lookup result ({}ms): \n{:?}", now.elapsed().as_millis(), r);

            let now = Instant::now();
            let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 5066702320566272);
            println!("lookup result ({}ms): \n{:?}", now.elapsed().as_millis(), r);

            let now = Instant::now();
            let filter = Filter {
                property: "Action",
                op: Operator::Equal,
                value: Value::String(String::from("Delete")),
            };
            let r: Result<Vec<Hero>, Error> = s.query("heroes", "Protocol", &filter);
            println!(
                "query result: {} ({}ms): \n",
                r.unwrap().len(),
                now.elapsed().as_millis()
            );

            let now = Instant::now();
            let r: Result<String, Error> = s.commit("heroes", "Rust-Test");
            println!(
                "commit result: {:?} ({}ms): \n",
                r,
                now.elapsed().as_millis()
            );
        }
        Err(msg) => error!("{}", msg),
    };
}
