use crate::authentication;
use crate::dotenv;
use log::error;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Auth {
    ApiKey(&'static str),
    JwtToken(&'static str),
}

impl Auth {
    pub fn to_query_url(&self) -> String {
        match self {
            Auth::ApiKey(k) => format!("key={}", k),
            Auth::JwtToken(t) => format!("access_token={}", t),
        }
    }

    pub fn create(kind: Auth, key: String) -> Result<Auth, &'static str> {
        match kind {
            Auth::ApiKey(k) => Ok(Auth::ApiKey(k)),
            Auth::JwtToken(_) => match jwt_token_login(Box::leak(key.into_boxed_str())) {
                Ok(token) => Ok(Auth::JwtToken(token)),
                Err(msg) => Err(msg),
            },
        }
    }
}

fn jwt_token_login(private_key: &'static str) -> Result<&'static str, &'static str> {
    match authentication::generate_jwt(authentication::Claim::new(), private_key) {
        Ok(token) => {
            // write to dot-env-file
            // temporary solution
            let mut dotenv = dotenv::Dotenv::new();
            dotenv.put(dotenv::KEY_JWT_TOKEN.to_string(), token.clone());
            if let Err(msg) = dotenv.write_to_file() {
                error!("{}", msg);
            }

            get_access_token(Box::leak(token.into_boxed_str()))
        }
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
