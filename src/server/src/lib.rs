#[macro_use]
extern crate slog;

use kvs::protocol::{GetResponse, RemoveResponse, Request, SetResponse};
use kvs::{KvsEngine, Result};

use std::net::{TcpListener, TcpStream, ToSocketAddrs};

mod pool;
mod server;

pub use server::KvsServer;

#[derive(Clone, Copy, Debug)]
pub enum EngineImpl {
    Kvs,
    Sled,
}

impl slog::Value for EngineImpl {
    fn serialize(&self, _rec: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        let s = match self {
            EngineImpl::Kvs => "KvsStore",
            EngineImpl::Sled => "SledEngine",
        };
        serializer.emit_str(key, &format!("{:?}", s))
    }
}
