use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
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

pub struct KvStore {
    store: HashMap<String, String>,
    path: PathBuf,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            store: HashMap::default(),
            path: PathBuf::from("./store.log"),
        }
    }
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.store.insert(key, value);
        Ok(())
    }
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.store.get(&key).cloned())
    }
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.store
            .remove(&key)
            .map_or_else(|| Err(From::from("Error removing key")), |_| Ok(()))
    }
    pub fn open(path: &Path) -> Result<Self> {
        Ok(KvStore {
            store: HashMap::default(),
            path: PathBuf::from(path),
        })
    }
}
