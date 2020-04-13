use super::converter::deserialize_query_result;
use super::{Error, ReadConsistency, ResponseError};
use http::StatusCode;
use reqwest::blocking;
use serde::de::DeserializeOwned;
use serde_json::Value;

const QUERY_JSON: &'static str = r#"{
    "partitionId": { "namespaceId": "{namespace}" },
    "readOptions": { "readConsistency": "{readConsistency}" },
    "query": { "kind": { "name": "{kind}", },
      "filter": {
        "propertyFilter": {
            "property": {
              "name": "Action",
            },
            "op" : "EQUAL",
            "value": {
              "stringValue":"GetByID",
            }
        }
      }
    }
}"#;

/*
{
  "partitionId": { "namespaceId": "{namespace}" },
  "readOptions": { "readConsistency": "{readConsistency}" },
  "query": { "kind": { "name": "{kind}", },
    "filter": {
      "propertyFilter": {
          "property": {
            "name": "Action",
          },
          "op" : "EQUAL",
          "value": {
            "stringValue":"List",
          }
      }
    }
  }
}

*/

fn create_query_json(namespace: &str, kind: &str) -> String {
  QUERY_JSON
    .replace("{readConsistency}", ReadConsistency::Eventual.to_string())
    .replace("{namespace}", namespace)
    .replace("{kind}", kind)
    .replace("\n", "")
}

pub fn query<D: DeserializeOwned>(
  client: &blocking::Client,
  auth_query_str: &str,
  project: &str,
  namespace: &str,
  kind: &str,
) -> Result<Vec<D>, Error> {
  let url = format!(
    "https://datastore.googleapis.com/v1/projects/{}:runQuery?{}",
    project, auth_query_str
  );
  let lookup_json = create_query_json(namespace, kind);
  let resp = client.post(&url).body(lookup_json).send()?;

  if resp.status().as_u16() == StatusCode::OK.as_u16() {
    let v = resp.json::<Value>().unwrap();
    return Ok(deserialize_query_result(&v)?);
  } else {
    Err(resp.json::<ResponseError>()?.error)
  }
}
