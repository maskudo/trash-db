use crate::{
    commands::{KvsCommands, KvsResponse},
    engines::KvsEngine,
    thread_pool::ThreadPool,
    KvError, MESSAGE_SIZE,
};
use log::info;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
};

pub struct KvServer<E: KvsEngine, T: ThreadPool> {
    threadpool: T,
    engine: E,
}

impl<E: KvsEngine, T: ThreadPool> KvServer<E, T> {
    pub fn new(engine: E, pool: T) -> Self {
        Self {
            engine,
            threadpool: pool,
        }
    }
    pub fn run(&mut self, addr: &str) -> crate::Result<()> {
        let listener = TcpListener::bind(&addr)?;
        info!("Listening on {}", addr);
        for stream in listener.incoming() {
            info!("Connection established");
            let stream = stream.unwrap();
            let kvs = self.engine.clone();
            self.threadpool.spawn(move || {
                Self::handle_connection(kvs, stream).unwrap();
            })
        }
        info!("Connection closed");
        Ok(())
    }
    fn handle_connection(kvs: E, mut stream: TcpStream) -> crate::Result<()> {
        let command = Self::get_command(&mut stream)?;
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

    fn get_command(stream: &mut TcpStream) -> crate::Result<KvsCommands> {
        let mut buffer = vec![];
        let mut bytes = [0; MESSAGE_SIZE];
        loop {
            let bytes_read = stream.read(&mut bytes).unwrap();
            buffer.extend_from_slice(&bytes);
            if bytes_read < MESSAGE_SIZE {
                break;
            }
        }

        let content = str::from_utf8(&mut buffer)
            .unwrap()
            .trim_matches(char::from(0));
        let command: KvsCommands = serde_json::from_str(content).unwrap();
        Ok(command)
    }
}
