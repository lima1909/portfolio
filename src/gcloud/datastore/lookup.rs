use crate::gcloud::auth::Auth;
use log::info;
use reqwest::blocking;

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
) {
    let url = format!(
        "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
        project,
        auth.to_query_url()
    );
    let lookup_json = lookup_json(namespace, kind, &id.to_string());
    let res = client.post(&url).body(lookup_json).send().unwrap();
    info!("response status-code: {}", res.status());
    info!("response: {}", &res.text().unwrap()[..50]);
}
