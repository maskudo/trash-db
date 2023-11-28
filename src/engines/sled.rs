use super::KvsEngine;
use crate::KvError;
use sled::Db;
use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct SledKvsEngine(Db);

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> crate::Result<Option<String>> {
        let tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }
    fn set(&mut self, key: String, value: String) -> crate::Result<()> {
        let tree = &self.0;
        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }
    fn remove(&mut self, key: String) -> crate::Result<()> {
        let tree = &self.0;
        tree.remove(key)?.ok_or(KvError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> crate::Result<Self> {
        Ok(SledKvsEngine(sled::open(path.into())?))
    }
}

impl Default for SledKvsEngine {
    fn default() -> Self {
        Self::open(
            env::current_dir()
                .expect("Error getting current dir")
                .as_path(),
        )
        .expect("Error creating SledKvStore")
    }
}
