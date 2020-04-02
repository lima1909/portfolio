use crate::gcloud::auth::Auth;
use log::info;

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

fn lookup_json(namespace: String, kind: String, id: String) -> String {
    LOOKUP_JSON
        .replace("{readConsistency}", ReadConsistency::Eventual.to_string())
        .replace("{namespace}", &namespace)
        .replace("{kind}", &kind)
        .replace("{id}", &id)
        .replace("\n", "")
}

pub struct Datastore<'a, T: Auth<'a>> {
    project: String,
    auth: &'a T,
    client: reqwest::blocking::Client,
}

impl<'a, T> Datastore<'a, T>
where
    T: Auth<'a>,
{
    pub fn new(project: String, auth: &'a T) -> Datastore<'a, T> {
        Datastore {
            project: project,
            auth: auth,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn lookup(self, namespace: String, kind: String, id: i128) {
        let url = format!(
            "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
            self.project,
            self.auth.to_query_url() // format!("access_token={}", at.access_token)
        );
        let lookup_json = lookup_json(namespace, kind, id.to_string());
        let res = self.client.post(&url).body(lookup_json).send().unwrap();
        info!("response status-code: {}", res.status());
        info!("response: {}", &res.text().unwrap()[..50]);

        // let a = async {
        //     let client = reqwest::Client::new();
        //     let res = client.post(&url).body(json).send().await;

        //     // Ok(res);
        // };

        // let res: executor::block_on(a);
    }
}
