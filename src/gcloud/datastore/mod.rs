pub mod converter;
pub mod lookup;

use http::StatusCode;
use reqwest::blocking;
use reqwest::{self};
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

impl From<reqwest::Error> for ResponseError {
    fn from(err: reqwest::Error) -> Self {
        let status = match err.status() {
            Some(s) => s.as_u16(),
            None => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };

        let mut url = "no url available";
        if let Some(u) = err.url() {
            url = u.as_str();
        }
        ResponseError {
            error: Error {
                code: status,
                message: format!("{} (url: {})", err, url),
                status: status.to_string(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Entity {
    key: Key,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Key {
    #[serde(rename(deserialize = "partitionId"))]
    partition_id: PartitionId,
    path: Vec<Path>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PartitionId {
    #[serde(rename(deserialize = "projectId"))]
    project_id: String,
    #[serde(rename(deserialize = "namespaceId"))]
    namespace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Path {
    kind: String,
    id: String,
}

impl Entity {
    fn to_string(self) -> String {
        format!(
            "project: {} namespace {} kind: {}, id: {}",
            self.key.partition_id.project_id,
            self.key.partition_id.namespace_id,
            self.key.path.get(0).unwrap().kind,
            self.key.path.get(0).unwrap().id
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
        let q = a.to_query_url();
        let s = Datastore::new("project-not-exist", &q);
        let r: Result<NotUsed, ResponseError> = s.lookup("ns", "kind", 42);
        match r {
            Err(e) => assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), e.error.code),
            Ok(_) => (),
        }
    }
}
