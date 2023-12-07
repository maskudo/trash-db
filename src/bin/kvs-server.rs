use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
};

use clap::{Parser, ValueEnum};
use log::info;
use serde::{Deserialize, Serialize};
use trash_db::{
    engines::{kvstore::KvStore, sled::SledKvsEngine, KvsEngine},
    server::KvServer,
    thread_pool::{rayon::RayonThreadPool, ThreadPool},
    Result,
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
    let selection_engine = cli.engine;
    let current_engine = get_current_engine()?;
    let engine = handle_engine_selection(current_engine, selection_engine)?;
    let addr = cli.addr;
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {:?}", engine);
    let pool = RayonThreadPool::new(num_cpus::get())?;
    match engine {
        Engine::Kvs => run_with_engine(KvStore::default(), pool, &addr),
        Engine::Sled => run_with_engine(SledKvsEngine::default(), pool, &addr),
    }
}

fn run_with_engine<E: KvsEngine, T: ThreadPool>(
    engine: E,
    pool: T,
    addr: &str,
) -> crate::Result<()> {
    let mut server = KvServer::new(engine, pool);
    server.run(addr)
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

fn handle_engine_selection(
    current_engine: Option<Engine>,
    selection_engine: Option<Engine>,
) -> crate::Result<Engine> {
    match (current_engine, selection_engine) {
        (Some(cur_engine), None) => Ok(cur_engine),
        (Some(cur_engine), Some(engine)) => {
            if cur_engine != engine {
                return Err(From::from(format!(
                    "Illegal engine selection. Current engine: {:?}",
                    cur_engine
                )));
            }
            Ok(engine)
        }
        (None, Some(engine)) => {
            let content = serde_json::to_string(&engine)?;
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(".engine")?;
            file.write(content.as_bytes())?;
            Ok(engine)
        }
        (None, None) => {
            let engine = Engine::Kvs;
            let content = serde_json::to_string(&engine)?;
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(".engine")?;
            file.write(content.as_bytes())?;
            Ok(engine)
        }
    }
}
