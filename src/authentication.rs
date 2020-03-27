use chrono::prelude::*;
use jsonwebtoken;
use serde::{Deserialize, Serialize};

const URL_AUTH: &'static str = "https://oauth2.googleapis.com/token";

// https://developers.google.com/identity/protocols/oauth2/service-account
//
// iss	The email address of the service account.
// scope	A space-delimited list of the permissions that the application requests.
// aud	A descriptor of the intended target of the assertion. When making an access token request this value is always https://oauth2.googleapis.com/token.
// exp	The expiration time of the assertion, specified as seconds since 00:00:00 UTC, January 1, 1970. This value has a maximum of 1 hour after the issued time.
// iat	The time the assertion was issued, specified as seconds since 00:00:00 UTC, January 1, 1970.

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Claim {
    iss: String,   // client_email
    scope: String, // scope: https://www.googleapis.com/auth/devstorage.read_only
    aud: String,
    iat: i64,
    exp: i64,
}

impl Claim {
    pub fn new() -> Claim {
        Claim {
            iss: "bucket@goheros-207118.iam.gserviceaccount.com".to_string(),
            scope: "https://www.googleapis.com/auth/devstorage.read_only".to_string(),
            aud: URL_AUTH.to_string(),
            iat: Utc::now().timestamp(),
            exp: Utc::now().timestamp() + chrono::Duration::minutes(1).num_seconds(),
        }
    }
}

pub fn generate_jwt(claim: Claim, private_key: &str) -> Result<String, String> {
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
