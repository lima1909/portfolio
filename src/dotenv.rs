use log::info;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};

const DEFAULT_FILE: &str = ".env";

pub struct Dotenv {
    store: HashMap<String, String>,
}

#[allow(dead_code)]
impl Dotenv {
    pub fn new() -> Dotenv {
        let mut dotenv = Dotenv {
            store: HashMap::new(),
        };

        match File::open(DEFAULT_FILE) {
            Ok(file) => {
                let reader = BufReader::new(file);
                reader.lines().for_each(|l| {
                    if let Ok(line) = l {
                        let kv: Vec<&str> = line.split('=').collect();
                        dotenv.store.insert(kv[0].to_string(), kv[1].to_string());
                        info!("rows: {} -> {}", kv[0], kv[1]);
                    }
                })
            }
            Err(msg) => {
                println!("could not open file '{}': {}", DEFAULT_FILE, msg);
            }
        };
        dotenv
    }

    pub fn write_to_file(&self) -> Result<()> {
        let mut file = OpenOptions::new().write(true).open(DEFAULT_FILE)?;
        let mut content = String::new();
        for (k, v) in &self.store {
            content.push_str(k);
            content.push_str("=");
            content.push_str(v);
            content.push_str("\n");
        }
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn put(&mut self, k: String, v: String) {
        self.store.insert(k, v);
    }

    pub fn get(&self, k: &String) -> Option<&String> {
        self.store.get(k)
    }

    pub fn get_as_bytes(&self, k: &String) -> Option<&[u8]> {
        if let Some(v) = self.get(k) {
            return Some(v.as_bytes());
        };
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dotenv_values() {
        let mut d = Dotenv::new();
        assert_eq!(None, d.get_as_bytes(&"blub".to_string()));

        d.put("foo".to_string(), "bar".to_string());
        assert_eq!(Some("bar".as_bytes()), d.get_as_bytes(&"foo".to_string()));
        assert_eq!(None, d.get_as_bytes(&"not found".to_string()));

        assert_eq!(Some(&"bar".to_string()), d.get(&"foo".to_string()));
        assert_eq!(None, d.get(&"not found".to_string()));
    }
}
