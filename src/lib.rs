use std::{
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Get {
        #[clap(value_parser)]
        key: String,
    },
    Set {
        #[clap(value_parser)]
        key: String,
        #[clap(value_parser)]
        value: String,
    },
    Rm {
        #[clap(value_parser)]
        key: String,
    },
}

// # TODO : Compaction
#[derive(Serialize, Deserialize)]
pub struct KvStore {
    store: HashMap<String, String>,
    #[serde(skip_serializing)]
    path: PathBuf,
}

impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStore {
    pub fn new() -> Self {
        let pathbuf = PathBuf::from("./store");
        let file = File::open(&pathbuf);
        let content = match file {
            Ok(file) => {
                let file_reader = BufReader::new(file);
                let content: HashMap<String, String> = match serde_json::from_reader(file_reader) {
                    Ok(content) => content,
                    Err(_) => HashMap::default(),
                };

                content
            }
            Err(_) => HashMap::default(),
        };

        KvStore {
            store: content,
            path: pathbuf,
        }
    }
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        self.store.insert(key, value);
        let content = serde_json::to_string(&self.store)?;
        write!(writer, "{}", content)?;
        Ok(())
    }
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.store.get(&key).cloned())
    }
    pub fn remove(&mut self, key: String) -> Result<()> {
        let res = self.store.remove(&key);
        match res {
            Some(_) => {}
            None => {
                println!("Key not found");
                return Err(From::from(""));
            }
        }
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        let content = serde_json::to_string(&self.store)?;
        write!(writer, "{}", content)?;
        Ok(())
    }
    pub fn open(path: &Path) -> Result<Self> {
        let mut pathbuf = PathBuf::from(path);
        pathbuf.push("store");
        let file = File::open(&pathbuf);
        let content = match file {
            Ok(file) => {
                let file_reader = BufReader::new(file);
                let content: HashMap<String, String> = serde_json::from_reader(file_reader)?;
                content
            }
            Err(_) => HashMap::default(),
        };

        Ok(KvStore {
            store: content,
            path: pathbuf,
        })
    }
}
