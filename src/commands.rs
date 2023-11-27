use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum KvsCommands {
    Get { key: String },
    Set { key: String, value: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KvsResponse {
    Ok(Option<String>),
    Err(String),
}
