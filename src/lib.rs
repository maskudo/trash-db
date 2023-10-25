use std::{
    borrow::BorrowMut,
    collections::HashMap,
    env,
    error::Error,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
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

#[derive(Clone, Copy, Debug)]
pub enum KvError {
    KeyNotFound,
}
impl Display for KvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvError::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

// # TODO : Compaction
#[derive(Debug)]
pub struct KvStore {
    store: HashMap<String, CommandPos>,
    path: PathBuf,
    writer: BufWriter<File>,
}

#[derive(Debug, Clone, Copy)]
struct CommandPos {
    pub pos: u64,
    pub len: u64,
}
impl CommandPos {
    fn new(pos: u64, len: u64) -> Self {
        CommandPos { pos, len }
    }
}

impl Default for KvStore {
    fn default() -> Self {
        Self::open(
            env::current_dir()
                .expect("Error getting current dir")
                .as_path(),
        )
        .expect("Error creating KvStore")
    }
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let writer = &mut self.writer;
        let current_pos = writer
            .seek(SeekFrom::End(0))
            .expect("Error getting current writer position");
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();
        let key_length = key_bytes.len() as u32;
        let value_length = value_bytes.len() as u32;
        writer.write_all(&key_length.to_le_bytes())?;
        writer.write_all(&value_length.to_le_bytes())?;
        writer.write_all(key_bytes)?;
        writer.write_all(value_bytes)?;
        writer.flush()?;
        self.store.insert(
            key.clone(),
            CommandPos::new(current_pos, key_length as u64 + value_length as u64 + 8u64),
        );
        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let pos = self.store.get(&key);
        match pos {
            None => Ok(None),
            Some(&pos) => {
                let file = File::open(&self.path)?;
                let mut file_reader = BufReader::new(file);
                file_reader.seek(SeekFrom::Start(pos.pos))?;
                let chunk = &mut [0u8; 8];
                file_reader.borrow_mut().take(4 + 4).read_exact(chunk)?;
                let key_length = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let value_length = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                let mut key_bytes = vec![0u8; key_length as usize];
                let mut val_bytes = vec![0u8; value_length as usize];
                file_reader
                    .borrow_mut()
                    .take(key_length as u64)
                    .read_exact(&mut key_bytes)?;
                file_reader
                    .borrow_mut()
                    .take(value_length as u64)
                    .read_exact(&mut val_bytes)?;
                let val = String::from_utf8(val_bytes)?;
                Ok(Some(val))
            }
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.store.contains_key(&key) {
            println!("{}", KvError::KeyNotFound);
            return Err(From::from(""));
        }
        let writer = &mut self.writer;
        let key_bytes = key.as_bytes();
        let key_length = key_bytes.len() as u32;
        writer.write_all(&key_length.to_le_bytes())?;
        writer.write_all(&0u32.to_le_bytes())?;
        writer.write_all(key_bytes)?;
        writer.flush()?;
        self.store.remove(&key);
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut pathbuf = PathBuf::from(path);
        pathbuf.push("store");
        let file = File::open(&pathbuf);
        let mut hashmap: HashMap<String, CommandPos> = HashMap::default();
        let content = match file {
            Ok(file) => {
                let mut file_reader = BufReader::new(file);
                let chunk = &mut [0u8; 8];
                while let Ok(_) = file_reader.borrow_mut().take(4 + 4).read_exact(chunk) {
                    let current_pos = file_reader
                        .seek(SeekFrom::Current(0))
                        .expect("Error getting current position")
                        - 8;
                    let key_length = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let value_length = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let mut key_bytes = vec![0u8; key_length as usize];
                    let mut val_bytes = vec![0u8; value_length as usize];
                    file_reader
                        .borrow_mut()
                        .take(key_length as u64)
                        .read_exact(&mut key_bytes)?;
                    file_reader
                        .borrow_mut()
                        .take(value_length as u64)
                        .read_exact(&mut val_bytes)?;
                    let key = String::from_utf8(key_bytes)?;
                    if value_length == 0 {
                        let _ = hashmap.remove(&key);
                        continue;
                    }
                    hashmap.insert(
                        key,
                        CommandPos::new(
                            current_pos,
                            8u64 + key_length as u64 + value_length as u64,
                        ),
                    );
                }

                hashmap
            }
            Err(_) => HashMap::default(),
        };
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(pathbuf.as_path())?;
        let writer = BufWriter::new(file);

        Ok(KvStore {
            store: content,
            path: pathbuf,
            writer,
        })
    }
}
