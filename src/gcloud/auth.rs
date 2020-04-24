use crate::gcloud::Error;

use log::debug;
use serde::Serialize;
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
        Self { key }
    }
}

#[derive(Debug)]
pub struct JwtToken<T: AsRef<str>> {
    pub jwt_token: String,
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
        Self {
            jwt_token: String::new(),
            access_token,
        }
    }
}

impl JwtToken<String> {
    pub fn from_env_private_key<T: Serialize>(claim: T) -> Result<JwtToken<String>, String> {
        match env::var(ENV_PRIVATE_KEY) {
            Ok(pk) => {
                debug!("{}: {:?}", ENV_PRIVATE_KEY, &pk[..50]);
                jwt_token_login(&pk, claim)
            }
            Err(msg) => {
                let err_msg = format!("could not read env {}: {}", ENV_PRIVATE_KEY, msg);
                Err(err_msg)
            }
        }
    }
}

fn jwt_token_login<T: Serialize>(
    private_key: impl AsRef<str>,
    claim: T,
) -> Result<JwtToken<String>, String> {
    match create_jwt_token(claim, private_key.as_ref()) {
        Ok(jwt_token) => match get_access_token(&jwt_token) {
            Ok(access_token) => Ok(JwtToken {
                jwt_token: jwt_token,
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

pub fn create_jwt_token<T: Serialize>(claim: T, private_key: &str) -> Result<String, String> {
    match jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes()) {
        Ok(pk) => {
            match jsonwebtoken::encode(
                &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
                &claim,
                &pk,
            ) {
                Ok(token) => Ok(token),
                Err(msg) => Err(format!("err by create jwt-token: {}", msg)),
            }
        }
        Err(msg) => Err(format!("err by read private key: {}", msg)),
    }
}
