pub mod lookup;
use crate::gcloud::auth::Auth;

use reqwest::blocking;

pub struct Datastore<'a, T: Auth<'a>> {
    project: &'a str,
    auth: &'a T,
    client: blocking::Client,
}

impl<'a, T> Datastore<'a, T>
where
    T: Auth<'a>,
{
    pub fn new(project: &'a str, auth: &'a T) -> Datastore<'a, T> {
        Datastore {
            project: project,
            auth: auth,
            client: blocking::Client::new(),
        }
    }

    pub fn lookup(self, namespace: &str, kind: &str, id: i128) {
        lookup::lookup(self.client, self.auth, self.project, namespace, kind, id)
    }
}
