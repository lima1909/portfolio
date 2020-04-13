pub mod converter;
pub mod lookup;
pub mod query;

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

    pub fn lookup<D>(&self, namespace: &str, kind: &str, id: i128) -> Result<D, Error>
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

    pub fn query<D>(&self, namespace: &str, kind: &str) -> Result<D, Error>
    where
        D: DeserializeOwned,
    {
        query::query(
            &self.client,
            &self.auth_query_str,
            &self.project,
            namespace,
            kind,
        )
    }
}

enum ReadConsistency {
    ReadConsistencyUnspecidied,
    Strong,
    Eventual,
}

impl ReadConsistency {
    #[allow(dead_code)]
    fn from_string(from: &str) -> Self {
        match from {
            "READ_CONSISTENCY_UNSPECIFIED" => ReadConsistency::ReadConsistencyUnspecidied,
            "STRONG" => ReadConsistency::Strong,
            _ => ReadConsistency::Eventual,
        }
    }

    fn to_string(&self) -> &str {
        match self {
            ReadConsistency::ReadConsistencyUnspecidied => "READ_CONSISTENCY_UNSPECIFIED",
            ReadConsistency::Strong => "STRONG",
            ReadConsistency::Eventual => "EVENTUAL",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseError {
    error: Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub code: u16,
    pub message: String,
    pub status: String,
}

impl Error {
    fn new(status_code: StatusCode, msg: String) -> Self {
        Error {
            code: status_code.as_u16(),
            message: msg,
            status: status_code.canonical_reason().unwrap().to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON_ERROR: {} (l{} : c{})", err, err.line(), err.column()),
        )
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let status = match err.status() {
            Some(s) => s.as_u16(),
            None => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };

        let url = match err.url() {
            Some(u) => u.as_str(),
            None => "no url available",
        };

        Error {
            code: status,
            message: format!("{} (url: {})", err, url),
            status: status.to_string(),
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
    use crate::gcloud::auth::{ApiKey, Auth, JwtToken};
    use http::StatusCode;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    struct NotUsed {}

    #[test]
    fn datastore_lookup_error_unauthorized_401() {
        let a = ApiKey::create("invalid-auth-key").unwrap();
        let q = a.to_query_url();
        let s = Datastore::new("project-not-exist", &q);
        let r: Result<NotUsed, Error> = s.lookup("ns", "kind", 42);
        match r {
            Err(e) => assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), e.code),
            Ok(_) => (),
        }
    }

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

    #[test]
    fn datastore_lookup_found() {
        let a = JwtToken::from_env_private_key().unwrap();
        let q = a.to_query_url();
        let s = Datastore::new("goheros-207118", &q);
        let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 5066702320566272);
        assert!(r.is_ok());
        let hero: Hero = r.unwrap();
        assert_eq!(2, hero.hero_id);
        assert_eq!("GetByID", hero.action);
    }

    #[test]
    fn datastore_lookup_missing() {
        let a = JwtToken::from_env_private_key().unwrap();
        let q = a.to_query_url();
        let s = Datastore::new("goheros-207118", &q);
        let r: Result<Hero, Error> = s.lookup("heroes", "Protocol", 42);
        assert!(r.is_err());
        let err: Error = r.unwrap_err();
        assert_eq!(404, err.code);
        assert_eq!("Not Found", err.status);
    }
}
