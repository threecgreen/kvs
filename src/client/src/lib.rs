use kvs::protocol::{GetResponse, RemoveResponse, Request, SetResponse};
use kvs::{Error, Result};

use std::io;
use std::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Client> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        bincode::serialize_into(&self.stream, &Request::Get { key })?;
        match bincode::deserialize_from(&self.stream)? {
            GetResponse::Ok(o) => Ok(o),
            GetResponse::Err(msg) => Err(Error::Server { msg }),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        bincode::serialize_into(&self.stream, &Request::Set { key, value })?;
        match bincode::deserialize_from(&self.stream)? {
            SetResponse::Ok(()) => Ok(()),
            SetResponse::Err(msg) => Err(Error::Server { msg }),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        bincode::serialize_into(&self.stream, &Request::Remove { key })?;
        match bincode::deserialize_from(&self.stream)? {
            RemoveResponse::Ok(()) => Ok(()),
            RemoveResponse::Err(msg) => Err(Error::Server { msg }),
        }
    }
}
