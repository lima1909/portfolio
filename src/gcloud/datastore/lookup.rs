use super::{Json, ResponseError};
use crate::gcloud::auth::Auth;
use http::StatusCode;
use reqwest::blocking;

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

fn lookup_json(namespace: &str, kind: &str, id: &str) -> String {
    LOOKUP_JSON
        .replace("{readConsistency}", ReadConsistency::Eventual.to_string())
        .replace("{namespace}", namespace)
        .replace("{kind}", kind)
        .replace("{id}", id)
        .replace("\n", "")
}

pub fn lookup<'a, T: Auth<'a>>(
    client: blocking::Client,
    auth: &'a T,
    project: &str,
    namespace: &str,
    kind: &str,
    id: i128,
) -> Result<Json, ResponseError> {
    let url = format!(
        "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
        project,
        auth.to_query_url()
    );
    let lookup_json = lookup_json(namespace, kind, &id.to_string());
    let res = client.post(&url).body(lookup_json).send().unwrap();

    if res.status().as_u16() == StatusCode::OK.as_u16() {
        match res.text() {
            Ok(json) => Ok(Json { json: json }),
            Err(msg) => Err(ResponseError::new_internal_server_error(
                msg.to_string(),
                "error read response lookup body",
            )),
        }
    } else {
        match res.json::<ResponseError>() {
            Ok(err) => Err(err),
            Err(msg) => Err(ResponseError::new_internal_server_error(
                msg.to_string(),
                "error by deserialize json-error-result",
            )),
        }
    }
}
