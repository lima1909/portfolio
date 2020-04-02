pub mod lookup;
use crate::gcloud::auth::Auth;

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

    pub fn lookup(self, namespace: &str, kind: &str, id: i128) {
        lookup::lookup(self.client, self.auth, &self.project, namespace, kind, id)
    }
}
