use clap::Parser;
use trash_db::Commands;
use trash_db::{Cli, KvStore};

fn main() {
    let cli = Cli::parse();
    let mut kvs = KvStore::new();
    match &cli.command {
        Commands::Get { key } => {
            unimplemented!("unimplemented")
        }
        Commands::Set { key, value } => {
            unimplemented!("unimplemented")
        }
        Commands::Rm { key } => {
            unimplemented!("unimplemented")
        }
    }
}
