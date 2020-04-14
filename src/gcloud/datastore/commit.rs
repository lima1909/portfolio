use super::{Error, ResponseError};

use http::StatusCode;
use reqwest::blocking;
use serde_json::Value;

pub fn transaction(
    client: &blocking::Client,
    auth_query_str: &str,
    project: &str,
) -> Result<String, Error> {
    let url = format!(
        "https://datastore.googleapis.com/v1/projects/{}:beginTransaction?{}",
        project, auth_query_str
    );
    let resp = client.post(&url).body("").send()?;

    if resp.status().as_u16() == StatusCode::OK.as_u16() {
        let v = resp.json::<Value>().unwrap();
        let trans = v.get("transaction").unwrap();
        return Ok(trans.as_str().unwrap().to_string());
    } else {
        Err(resp.json::<ResponseError>()?.error)
    }
}

pub fn commit(
    client: &blocking::Client,
    auth_query_str: &str,
    project: &str,
) -> Result<String, Error> {
    transaction(&client, &auth_query_str, &project)
}
