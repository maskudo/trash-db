use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::str::from_utf8;
use trash_db::commands::{KvsCommands, KvsResponse};
use trash_db::{Result, MESSAGE_SIZE};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, default_value_t = format!("127.0.0.1:4000"), global=true)]
    addr: String,
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
    let addr = cli.addr;
    let mut stream = TcpStream::connect(addr)?;

    let content = match &cli.command {
        Commands::Get { key } => serde_json::to_string(&KvsCommands::Get {
            key: key.to_string(),
        })
        .unwrap(),
        Commands::Set { key, value } => serde_json::to_string(&KvsCommands::Set {
            key: key.to_string(),
            value: value.to_string(),
        })
        .unwrap(),
        Commands::Rm { key } => serde_json::to_string(&KvsCommands::Rm {
            key: key.to_string(),
        })
        .unwrap(),
    };
    stream.write_all(content.as_str().as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut buffer = vec![];
    let mut bytes = [0; MESSAGE_SIZE];
    loop {
        let bytes_read = stream.read(&mut bytes).unwrap();
        buffer.extend_from_slice(&bytes);
        if bytes_read < MESSAGE_SIZE {
            break;
        }
    }
    let res = from_utf8(&mut buffer).unwrap().trim_matches(char::from(0));
    let res: KvsResponse = serde_json::from_str(res)?;
    match res {
        KvsResponse::Ok(Some(res)) => {
            println!("{}", res);
        }
        KvsResponse::Ok(None) => {}
        KvsResponse::Err(e) => {
            if let Commands::Get { key: _ } = &cli.command {
                println!("{}", e);
            } else {
                eprintln!("{}", e);
                exit(1);
            }
        }
    };
    Ok(())
}
