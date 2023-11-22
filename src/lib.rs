use std::{error::Error, fmt::Display};
pub mod commands;
pub mod engines;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

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

impl Error for KvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
