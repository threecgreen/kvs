#[macro_use]
extern crate slog;

use kvs::Result;

mod pool;
mod server;

pub use server::KvsServer;
pub use pool::{NaiveThreadPool, ThreadPool};

#[derive(Clone, Copy, Debug)]
pub enum EngineImpl {
    Kvs,
    Sled,
}

impl slog::Value for EngineImpl {
    fn serialize(
        &self,
        _rec: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        let s = match self {
            EngineImpl::Kvs => "KvsStore",
            EngineImpl::Sled => "SledEngine",
        };
        serializer.emit_str(key, &format!("{:?}", s))
    }
}
