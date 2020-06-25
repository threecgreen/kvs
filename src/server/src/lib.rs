#[macro_use]
extern crate slog;

use kvs::Result;

mod pool;
mod server;

pub use pool::{NaiveThreadPool, ThreadPool};
pub use server::KvsServer;

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
            EngineImpl::Kvs => "kvs",
            EngineImpl::Sled => "sled",
        };
        serializer.emit_str(key, &format!("{:?}", s))
    }
}
