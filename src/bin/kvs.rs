use std::process::exit;

use clap::Parser;
use trash_db::{Cli, KvStore, Result};
use trash_db::{Commands, KvError};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut kvs = KvStore::new();
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
