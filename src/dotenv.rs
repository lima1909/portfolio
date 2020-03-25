use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

const DEFAULT_FILE: &str = ".env";

pub struct Dotenv {
    store: HashMap<String, String>,
}

#[allow(dead_code)]
impl Dotenv {
    pub fn new() -> Dotenv {
        Dotenv {
            store: HashMap::new(),
        }
    }

    pub fn load(&mut self) {
        match File::open(DEFAULT_FILE) {
            Ok(file) => {
                let buf = BufReader::new(file).lines();
                buf.for_each(|l| {
                    if let Ok(line) = l {
                        let kv: Vec<&str> = line.split('=').collect();
                        self.store.insert(kv[0].to_string(), kv[1].to_string());
                        info!("rows: {} -> {}", kv[0], kv[1]);
                    }
                })
            }
            Err(msg) => {
                println!("could not open file '{}': {}", DEFAULT_FILE, msg);
                return;
            }
        }
    }

    pub fn put(&mut self, k: String, v: String) {
        self.store.insert(k, v);
    }

    pub fn get(&self, k: &String) -> Option<&String> {
        self.store.get(k)
    }

    pub fn get_as_bytes(&self, k: &String) -> &[u8] {
        match self.store.get(k) {
            Some(v) => return v.as_bytes(),
            None => return "".as_bytes(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dotenv_values() {
        let mut d = Dotenv::new();
        assert_eq!("".as_bytes(), d.get_as_bytes(&"blub".to_string()));

        d.put("foo".to_string(), "bar".to_string());
        assert_eq!("bar".as_bytes(), d.get_as_bytes(&"foo".to_string()));
        assert_eq!("".as_bytes(), d.get_as_bytes(&"not found".to_string()));

        assert_eq!(Some(&"bar".to_string()), d.get(&"foo".to_string()));
        assert_eq!(None, d.get(&"not found".to_string()));
    }
}
