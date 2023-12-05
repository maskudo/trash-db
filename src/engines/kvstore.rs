use super::KvsEngine;
use crate::KvError;
use crate::Result;
use std::sync::RwLock;
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct KvStore {
    store: Arc<RwLock<HashMap<String, CommandPos>>>,
    path: PathBuf,
    write_agent: Arc<Mutex<WriteAgent>>,
}
impl Clone for KvStore {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            path: self.path.clone(),
            write_agent: self.write_agent.clone(),
        }
    }
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

impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.write_agent.lock().unwrap().set(key, value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let r = self.store.read().unwrap();
        let pos = r.get(&key);
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

    fn remove(&self, key: String) -> Result<()> {
        self.write_agent.lock().unwrap().remove(key)
    }
}

impl KvStore {
    pub fn open(path: &Path) -> Result<Self> {
        let mut pathbuf = PathBuf::from(path);
        pathbuf.push(".store");
        let file = File::open(&pathbuf);
        let mut hashmap: HashMap<String, CommandPos> = HashMap::default();
        let mut stale_bytes = 0;
        let content = match file {
            Ok(file) => {
                let mut file_reader = BufReader::new(file);
                let chunk = &mut [0u8; 8];
                while file_reader
                    .borrow_mut()
                    .take(4 + 4)
                    .read_exact(chunk)
                    .is_ok()
                {
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
                        let value = hashmap.remove(&key);
                        if let Some(value) = value {
                            stale_bytes += value.len + key_length as u64 + 4 + 4;
                        }
                        continue;
                    }
                    let res = hashmap.insert(
                        key,
                        CommandPos::new(
                            current_pos,
                            8u64 + key_length as u64 + value_length as u64,
                        ),
                    );
                    if let Some(value) = res {
                        stale_bytes += value.len + key_length as u64 + 4 + 4;
                    }
                }

                hashmap
            }
            Err(_) => HashMap::default(),
        };
        let store = Arc::new(RwLock::new(content));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(pathbuf.as_path())?;
        let writer = BufWriter::new(file);
        let writer = WriteAgent {
            index: store.clone(),
            writer,
            stale_bytes,
            path: pathbuf.clone(),
        };
        let writer = Arc::new(Mutex::new(writer));
        Ok(KvStore {
            store: store.clone(),
            path: pathbuf,
            write_agent: writer,
        })
    }
}

#[derive(Debug)]
struct WriteAgent {
    index: Arc<RwLock<HashMap<String, CommandPos>>>,
    path: PathBuf,
    writer: BufWriter<File>,
    stale_bytes: u64,
}

impl WriteAgent {
    pub fn set(&mut self, key: String, value: String) -> crate::Result<()> {
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
        let res = self.index.write().unwrap().insert(
            key.clone(),
            CommandPos::new(current_pos, key_length as u64 + value_length as u64 + 8u64),
        );
        if let Some(value) = res {
            self.stale_bytes += value.len + key_length as u64 + 4 + 4;
        }
        if self.stale_bytes >= COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> crate::Result<()> {
        let r = self.index.read().unwrap();
        let value = r.get(&key);
        if value.is_none() {
            // println!("{}", KvError::KeyNotFound);
            return Err(From::from(KvError::KeyNotFound));
        }
        let writer = &mut self.writer;
        let key_bytes = key.as_bytes();
        let key_length = key_bytes.len() as u32;
        writer.write_all(&key_length.to_le_bytes())?;
        writer.write_all(&0u32.to_le_bytes())?;
        writer.write_all(key_bytes)?;
        writer.flush()?;
        self.stale_bytes += key_bytes.len() as u64 + value.unwrap().len + 4 + 4;
        drop(r);
        if self.stale_bytes >= COMPACTION_THRESHOLD {
            self.compact()?;
        }
        self.index.write().unwrap().remove(&key);
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut path = self.path.clone();
        path.pop();
        path.push("temp_file");
        let temp_file = OpenOptions::new().create(true).append(true).open(&path)?;

        let mut writer = BufWriter::new(temp_file);
        let mut reader = BufReader::new(file);
        for item in self.index.write().unwrap().values_mut() {
            reader.borrow_mut().seek(SeekFrom::Start(item.pos))?;
            let mut bytes = vec![0u8; item.len as usize];
            reader.borrow_mut().take(item.len).read_exact(&mut bytes)?;
            let pos = writer.seek(SeekFrom::End(0))?;
            writer.write_all(&bytes)?;
            item.pos = pos;
            writer.flush()?;
        }

        fs::rename(path.as_path(), self.path.as_path())?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(PathBuf::from(&self.path))?;
        let writer = BufWriter::new(file);
        self.writer = writer;
        self.stale_bytes = 0;
        Ok(())
    }
}
