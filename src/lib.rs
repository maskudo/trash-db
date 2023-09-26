use std::{collections::HashMap, path::PathBuf};

use clap::{Parser, Subcommand};

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
            path: PathBuf::default(),
        }
    }
    pub fn set(&mut self, key: String, value: String) {
        unimplemented!()
    }
    pub fn get(&self, key: String) -> Option<String> {
        unimplemented!()
    }
    pub fn remove(&mut self, key: String) {
        unimplemented!()
    }
}
