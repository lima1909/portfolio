// use crate::gcloud::Store;
use crate::gcloud::auth::Auth;
// use futures::executor;
use log::info;
// use reqwest::blocking;
use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct readOptions {
    readConsistency: String,
}
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct partitionId {
    namespaceId: String,
}
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct path {
    kind: String,
    id: String,
}
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct key {
    partitionId: partitionId,
    path: path,
}
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct lookup {
    readOptions: readOptions,
    keys: Vec<key>,
}

fn create_lookup(namespace: String, kind: String, id: String) -> lookup {
    lookup {
        readOptions: readOptions {
            readConsistency: "EVENTUAL".to_owned(),
        },
        keys: vec![key {
            partitionId: partitionId {
                namespaceId: namespace,
            },
            path: path { kind: kind, id: id },
        }],
    }
}

impl lookup {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub struct Datastore<'a, T: Auth<'a>> {
    project: String,
    auth: &'a T,
}

impl<'a, T> Datastore<'a, T>
where
    T: Auth<'a>,
{
    pub fn new(project: String, auth: &'a T) -> Datastore<'a, T> {
        Datastore {
            project: project,
            auth: auth,
        }
    }

    pub fn lookup(self, namespace: String, kind: String, id: i128) {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "https://datastore.googleapis.com/v1/projects/{}:lookup?{}",
            self.project,
            self.auth.to_query_url() // format!("access_token={}", at.access_token)
        );
        let req_json = create_lookup(namespace, kind, id.to_string()).to_json();
        let res = client.post(&url).body(req_json).send().unwrap();
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
