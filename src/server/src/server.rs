use crate::pool::ThreadPool;

use kvs::{KvsEngine, Result, protocol::{GetResponse, SetResponse, RemoveResponse, Request}};
use std::net::{ToSocketAddrs, TcpListener, TcpStream};

#[derive(Debug)]
pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    inner: Inner<E>,
    pool: P,
}

#[derive(Clone, Debug)]
struct Inner<E: KvsEngine> {
    engine: E,
    log: slog::Logger,
}


impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(engine: E, log: &slog::Logger, pool: P) -> Self {
        Self {
            inner: Inner {
                engine,
                // TODO: add context here
                log: log.new(o!()),
            },
            pool,
        }
    }

    pub fn serve(&mut self, addr: impl ToSocketAddrs) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let inner = self.inner.clone();
                    self.pool.spawn(move || inner.handle_and_log(stream))
                }
                Err(e) => error!(self.inner.log, "Connection failed"; "error" => format!("{:?}", e)),
            }
        }
        Ok(())
    }
}

impl<E: KvsEngine> Inner<E> {
    fn handle_and_log(&self, stream: TcpStream) {
        if let Err(e) = self.handle_stream(stream) {
            error!(self.log, "Failed handling stream"; "error" => e);
        }
    }

    fn handle_stream(&self, stream: TcpStream) -> Result<()> {
        let timeout: std::time::Duration = std::time::Duration::new(30, 0);
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        match bincode::deserialize_from(&stream)? {
            Request::Get { key } => {
                info!(self.log, "Handling get request"; "key" => &key);
                let res = self.engine.get(key);
                bincode::serialize_into(stream, &GetResponse::from(res))
            }
            Request::Set { key, value } => {
                info!(self.log, "Handling set request"; "key" => &key, "value" => &value);
                let res = self.engine.set(key, value);
                bincode::serialize_into(stream, &SetResponse::from(res))
            }
            Request::Remove { key } => {
                info!(self.log, "Handling remove request"; "key" => &key);
                let res = self.engine.remove(key);
                bincode::serialize_into(stream, &RemoveResponse::from(res))
            }
        }?;
        Ok(())
    }
}
