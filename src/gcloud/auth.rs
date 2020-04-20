use crate::authentication;
use crate::gcloud::datastore::Error;

use log::debug;
use serde_json::Value;

use std::env;

const ENV_PRIVATE_KEY: &'static str = "PRIVATE_KEY";

pub trait Auth<T: AsRef<str>>: Sized {
    fn to_url_query(&self) -> String;
    fn new(key: T) -> Self;
}

#[derive(Debug)]
pub struct ApiKey<T: AsRef<str>> {
    pub key: T,
}

impl<T> Auth<T> for ApiKey<T>
where
    T: AsRef<str>,
{
    fn to_url_query(&self) -> String {
        format!("key={}", self.key.as_ref())
    }

    fn new(key: T) -> Self {
        ApiKey { key: key }
    }
}

#[derive(Debug)]
pub struct JwtToken<T: AsRef<str>> {
    pub jwt_token: &'static str,
    access_token: T,
}

impl<T> Auth<T> for JwtToken<T>
where
    T: AsRef<str>,
{
    fn to_url_query(&self) -> String {
        format!("access_token={}", self.access_token.as_ref())
    }

    fn new(access_token: T) -> Self {
        JwtToken {
            jwt_token: "",
            access_token: access_token,
        }
    }
}

impl JwtToken<String> {
    pub fn from_env_private_key() -> Result<JwtToken<String>, String> {
        match env::var(ENV_PRIVATE_KEY) {
            Ok(pk) => {
                debug!("{}: {:?}", ENV_PRIVATE_KEY, &pk[..50]);
                jwt_token_login(&pk)
            }
            Err(msg) => {
                let err_msg = format!("could not read env {}: {}", ENV_PRIVATE_KEY, msg);
                Err(err_msg)
            }
        }
    }
}

fn jwt_token_login(private_key: impl AsRef<str>) -> Result<JwtToken<String>, String> {
    match authentication::generate_jwt(authentication::Claim::new(), private_key.as_ref()) {
        Ok(jwt_token) => match get_access_token(&jwt_token) {
            Ok(access_token) => Ok(JwtToken {
                jwt_token: Box::leak(jwt_token.into_boxed_str()),
                access_token: access_token,
            }),
            Err(err) => Err(err.into()),
        },
        Err(msg) => Err(msg),
    }
}

fn get_access_token(jwt_token: impl AsRef<str>) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let json_resp = client
        .post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}",
            jwt_token.as_ref()
        ))
        .send()?;

    let v: Value = json_resp.json()?;
    let s = v.get("access_token").unwrap().as_str().unwrap();
    Ok(s.to_string())
}
