use clap::Parser;
use trash_db::Cli;
use trash_db::Commands;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Get { key } => {
            println!("getting {key}")
        }
        Commands::Set { key, value } => {
            println!("setting {key}: {value}")
        }
        Commands::Rm { key } => {
            println!("removing {key}")
        }
    }
}
