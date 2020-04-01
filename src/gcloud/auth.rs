use crate::authentication;
use serde::Deserialize;

pub trait Auth<'a>: Sized {
    fn to_query_url(&self) -> String;
    fn create(key: &'a str) -> Result<Self, &'static str>;
}

#[derive(Debug)]
pub struct ApiKey<'a> {
    pub key: &'a str,
}

impl<'a> Auth<'a> for ApiKey<'a> {
    fn to_query_url(&self) -> String {
        format!("key={}", self.key)
    }

    fn create(key: &'a str) -> Result<Self, &'static str> {
        Ok(ApiKey { key: key })
    }
}

#[derive(Debug)]
pub struct JwtToken<'a> {
    pub jwt_token: &'a str,
    access_token: &'a str,
}

impl<'a> Auth<'a> for JwtToken<'a> {
    fn to_query_url(&self) -> String {
        format!("access_token={}", self.access_token)
    }

    fn create(private_key: &'a str) -> Result<Self, &'static str> {
        jwt_token_login(private_key)
    }
}

fn jwt_token_login(private_key: &str) -> Result<JwtToken, &'static str> {
    match authentication::generate_jwt(authentication::Claim::new(), private_key) {
        Ok(jwt_token) => match get_access_token(Box::leak(jwt_token.into_boxed_str())) {
            Ok(access_token) => Ok(JwtToken {
                jwt_token: "",
                access_token: &access_token,
            }),
            Err(msg) => Err(msg),
        },
        Err(msg) => Err(Box::leak(msg.into_boxed_str())),
    }
}

fn get_access_token(jwt_token: &'static str) -> Result<&'static str, &'static str> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let client = reqwest::blocking::Client::new();
    let json_resp = client
        .post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}",
            jwt_token
        ))
        .send()
        .unwrap();

    let ar: TokenResponse = json_resp.json().unwrap();
    Ok(Box::leak(ar.access_token.into_boxed_str()))
}
