use crate::Result;

use serde::{Deserialize, Serialize};

/// A database command that can be send across the network
#[derive(Deserialize, Serialize, Debug)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Remove { key: String },
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SetResponse {
    Ok(()),
    Err(String),
}

impl From<Result<()>> for SetResponse {
    fn from(res: Result<()>) -> Self {
        match res {
            Ok(()) => SetResponse::Ok(()),
            Err(e) => SetResponse::Err(format!("{}", e)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum GetResponse {
    Ok(Option<String>),
    Err(String),
}

impl From<Result<Option<String>>> for GetResponse {
    fn from(res: Result<Option<String>>) -> Self {
        match res {
            Ok(v) => GetResponse::Ok(v),
            Err(e) => GetResponse::Err(format!("{}", e)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum RemoveResponse {
    Ok(()),
    Err(String),
}

impl From<Result<()>> for RemoveResponse {
    fn from(res: Result<()>) -> Self {
        match res {
            Ok(()) => RemoveResponse::Ok(()),
            Err(e) => RemoveResponse::Err(format!("{}", e)),
        }
    }
}
