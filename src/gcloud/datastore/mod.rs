pub mod converter;
pub mod lookup;

use http::StatusCode;
use reqwest::blocking;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{self};

pub struct Datastore<'a> {
    project: &'a str,
    auth_query_str: &'a str,
    client: blocking::Client,
}

impl<'a> Datastore<'a> {
    pub fn new(project: &'a str, auth_query_str: &'a str) -> Self {
        Datastore {
            project: project,
            auth_query_str: auth_query_str,
            client: blocking::Client::new(),
        }
    }

    pub fn lookup<D>(&self, namespace: &str, kind: &str, id: i128) -> Result<D, ResponseError>
    where
        D: DeserializeOwned,
    {
        lookup::lookup(
            &self.client,
            &self.auth_query_str,
            &self.project,
            namespace,
            kind,
            id,
        )
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

impl ResponseError {
    fn new_internal_server_error(msg: String, status: &str) -> Self {
        ResponseError {
            error: Error {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: msg,
                status: status.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        ResponseError::new_internal_server_error(
            format!("{} (l{} : c{})", err, err.line(), err.column()),
            format!("JSON_ERROR: ({:?})", err.classify()).as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::auth::{ApiKey, Auth};
    use http::StatusCode;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    struct NotUsed {}

    #[test]
    fn datastore_lookup_error_unauthorized_401() {
        let a = ApiKey::create("invalid-auth-key").unwrap();
        let q = Box::leak(a.to_query_url().clone().into_boxed_str());
        let s = Datastore::new("project-not-exist", q);
        let r: Result<NotUsed, ResponseError> = s.lookup("ns", "kind", 42);
        match r {
            Err(e) => assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), e.error.code),
            Ok(_) => (),
        }
    }
}
