pub mod lookup;
use crate::gcloud::auth::Auth;

use reqwest::blocking;
use serde::{Deserialize, Serialize};

pub struct Datastore<'a, T: Auth<'a>> {
    project: &'a str,
    auth: &'a T,
    client: blocking::Client,
}

impl<'a, T> Datastore<'a, T>
where
    T: Auth<'a>,
{
    pub fn new(project: &'a str, auth: &'a T) -> Datastore<'a, T> {
        Datastore {
            project: project,
            auth: auth,
            client: blocking::Client::new(),
        }
    }

    pub fn lookup(self, namespace: &str, kind: &str, id: i128) -> Result<Json, ResponseError> {
        lookup::lookup(self.client, self.auth, self.project, namespace, kind, id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseError {
    pub error: Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub code: u16,
    pub message: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Json {
    pub json: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::auth::{ApiKey, Auth};
    use http::StatusCode;

    #[test]
    fn datastore_lookup_error_unauthorized_401() {
        let a = ApiKey::create("invalid-auth-key").unwrap();
        let s = Datastore::new("project-not-exist", &a);
        match s.lookup("ns", "kind", 42) {
            Err(e) => assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), e.error.code),
            Ok(_) => (),
        }
    }
}
