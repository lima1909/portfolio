use super::ResponseError;
use crate::gcloud::auth::Auth;
use http::StatusCode;
use log::error;
use reqwest::blocking;
use serde::de::DeserializeOwned;
use serde_json::map::Map;
use serde_json::{Number, Value};

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

pub fn lookup<'a, D: DeserializeOwned, T: Auth<'a>>(
    client: blocking::Client,
    auth: &'a T,
    project: &str,
    namespace: &str,
    kind: &str,
    id: i128,
) -> Result<D, ResponseError> {
    let url = format!(
        "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
        project,
        auth.to_query_url()
    );
    let lookup_json = lookup_json(namespace, kind, &id.to_string());
    let res = client.post(&url).body(lookup_json).send().unwrap();

    if res.status().as_u16() == StatusCode::OK.as_u16() {
        match res.json::<Value>() {
            Ok(v) => Ok(convert_result(&v).unwrap()),
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

fn convert_result<T>(v: &Value) -> Option<T>
where
    T: DeserializeOwned,
{
    match v.get("found") {
        Some(found) => {
            match found
                .get(0)
                .unwrap()
                .get("entity")
                .unwrap()
                .get("properties")
            {
                Some(prop_map) => Some(serde_json::from_value(to_object(prop_map)).unwrap()),
                _ => {
                    error!("invalid value type");
                    None
                }
            }
        }
        None => None,
    }

    //     match v.get("missing") {
    //         Some(missing) => println!("\nfound:\n {:?} \n", missing),
    //         None => (),
    //     }
}

fn to_object(map: &Value) -> Value {
    let mut result_map = Map::new();
    let m = map.as_object().unwrap();
    for k in m.keys() {
        let datatype = m.get(k).unwrap();
        let dm = datatype.as_object().unwrap();
        for dk in dm.keys() {
            let v = dm.get(dk).unwrap();
            result_map.insert(k.to_string(), to_value(v.as_str().unwrap(), dk));
        }
    }
    Value::Object(result_map)
}

// still missing datatypes:
// https://cloud.google.com/datastore/docs/reference/data/rest/v1/projects/runQuery#Value
//
fn to_value(val: &str, datatype: &str) -> Value {
    match datatype {
        "nullValue" => Value::Null,
        "doubleValue" => Value::Number(Number::from_f64(val.parse().unwrap()).unwrap()),
        "integerValue" => {
            let v: isize = val.parse().unwrap();
            Value::Number(Number::from(v))
        }
        "booleanValue" => Value::Bool(val.parse().unwrap()),
        _ => Value::String(val.to_string()), // timestampValue | stringValue
    }
}
