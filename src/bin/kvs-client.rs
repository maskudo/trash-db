use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::process::exit;
use trash_db::engines::kvstore::KvStore;
use trash_db::engines::KvsEngine;
use trash_db::KvError;
use trash_db::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Serialize, Deserialize, Clone, Debug)]
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut kvs = KvStore::default();
    match &cli.command {
        Commands::Get { key } => match kvs.get(key.to_owned())? {
            Some(val) => println!("{val}"),
            None => {
                println!("{}", KvError::KeyNotFound);
                exit(0)
            }
        },
        Commands::Set { key, value } => {
            kvs.set(key.clone(), value.clone())?;
        }
        Commands::Rm { key } => {
            kvs.remove(key.clone())?;
        }
    }
    Ok(())
}
