use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
};

use clap::{Parser, ValueEnum};
use log::info;
use serde::{Deserialize, Serialize};
use trash_db::{
    commands::{KvsCommands, KvsResponse},
    engines::{kvstore::KvStore, sled::SledKvsEngine, KvsEngine},
    thread_pool::{naive::NaiveThreadPool, ThreadPool},
    KvError, Result, MESSAGE_SIZE,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = format!("127.0.0.1:4000"))]
    addr: String,
    #[arg(value_enum, long)]
    engine: Option<Engine>,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
enum Engine {
    Kvs,
    Sled,
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cli = Cli::parse();
    let engine = cli.engine;
    let current_engine = get_current_engine()?;
    let engine = match (current_engine, engine) {
        (Some(cur_engine), None) => cur_engine,
        (Some(cur_engine), Some(engine)) => {
            if cur_engine != engine {
                panic!(
                    "Incorrect engine selection. Current engine: {:?}",
                    cur_engine
                );
            }
            engine
        }
        (None, Some(engine)) => {
            let content = serde_json::to_string(&engine)?;
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(".engine")?;
            file.write(content.as_bytes())?;
            engine
        }
        (None, None) => {
            let engine = Engine::Kvs;
            let content = serde_json::to_string(&engine)?;
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(".engine")?;
            file.write(content.as_bytes())?;
            engine
        }
    };
    let addr = cli.addr;
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {:?}", engine);
    match engine {
        Engine::Kvs => run_with_engine(KvStore::default(), addr)?,
        Engine::Sled => run_with_engine(SledKvsEngine::default(), addr)?,
    };
    Ok(())
}

fn run_with_engine<E: KvsEngine>(engine: E, addr: String) -> Result<()> {
    let kvs = engine;
    let listener = TcpListener::bind(&addr)?;
    let thread_poool = NaiveThreadPool::new(8)?;
    info!("Listening on {}", addr);
    for stream in listener.incoming() {
        info!("Connection established");
        let stream = stream.unwrap();
        let kvs = kvs.clone();
        thread_poool.spawn(move || {
            handle_connection(kvs, stream).unwrap();
        })
    }
    info!("Connection closed");
    Ok(())
}

fn handle_connection<E: KvsEngine>(kvs: E, mut stream: TcpStream) -> crate::Result<()> {
    let command = get_command(&mut stream)?;
    info!("Command: {:?}", command);
    let response = match command {
        KvsCommands::Get { key } => match kvs.get(key.to_owned())? {
            Some(val) => KvsResponse::Ok(Some(val)),
            None => KvsResponse::Err(KvError::KeyNotFound.to_string()),
        },
        KvsCommands::Set { key, value } => match kvs.set(key.clone(), value.clone()) {
            Ok(()) => KvsResponse::Ok(None),
            Err(e) => KvsResponse::Err(e.to_string()),
        },
        KvsCommands::Rm { key } => match kvs.remove(key.clone()) {
            Ok(()) => KvsResponse::Ok(None),
            Err(e) => KvsResponse::Err(e.to_string()),
        },
    };
    stream
        .write_all(serde_json::to_string(&response).unwrap().as_bytes())
        .unwrap();
    stream.flush().unwrap();
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn get_command(stream: &mut TcpStream) -> Result<KvsCommands> {
    let mut buffer = vec![];
    let mut bytes = [0; MESSAGE_SIZE];
    loop {
        let bytes_read = stream.read(&mut bytes).unwrap();
        buffer.extend_from_slice(&bytes);
        if bytes_read < MESSAGE_SIZE {
            break;
        }
    }

    let content = from_utf8(&mut buffer).unwrap().trim_matches(char::from(0));
    let command: KvsCommands = serde_json::from_str(content).unwrap();
    Ok(command)
}

fn get_current_engine() -> Result<Option<Engine>> {
    let engine = fs::read_to_string(".engine");
    let engine = match engine {
        Ok(engine) => engine,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => return Ok(None),
            _ => return Err(Box::new(e)),
        },
    };
    let engine: Engine =
        serde_json::from_str(&engine).map_err(|val| (format!("Corrupted engine value {}", val)))?;
    Ok(Some(engine))
}
