use super::converter::deserialize_result;
use super::{Error, ResponseError};
use http::StatusCode;
use reqwest::blocking;
use serde::de::DeserializeOwned;
use serde_json::Value;

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

const LOOKUP_JSON: &'static str = r#"{
    "readOptions": { "readConsistency": "{readConsistency}" },
    "keys": [
      {
        "partitionId": { "namespaceId": "{namespace}" },
        "path": [
          {
            "kind": "{kind}",
            "id": "{id}"
          }
        ]
      }
    ]
}"#;

fn create_lookup_json(namespace: &str, kind: &str, id: &str) -> String {
    LOOKUP_JSON
        .replace("{readConsistency}", ReadConsistency::Eventual.to_string())
        .replace("{namespace}", namespace)
        .replace("{kind}", kind)
        .replace("{id}", id)
        .replace("\n", "")
}

pub fn lookup<D: DeserializeOwned>(
    client: &blocking::Client,
    auth_query_str: &str,
    project: &str,
    namespace: &str,
    kind: &str,
    id: i128,
) -> Result<D, Error> {
    let url = format!(
        "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
        project, auth_query_str
    );
    let lookup_json = create_lookup_json(namespace, kind, &id.to_string());
    let resp = client.post(&url).body(lookup_json).send()?;

    if resp.status().as_u16() == StatusCode::OK.as_u16() {
        let v = resp.json::<Value>().unwrap();
        return Ok(deserialize_result(&v)?);
    } else {
        Err(resp.json::<ResponseError>()?.error)
    }
}
