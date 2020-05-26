#[macro_use]
extern crate slog;

use kvs::{KvStore, SledEngine};
use kvs_server::{EngineImpl, KvsServer};

use clap::{App, Arg};
use slog::Drain;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Instantiate log
    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let version = env!("CARGO_PKG_VERSION");
    let default_addr = "127.0.0.1:4000";

    let args = App::new("kvs-server")
        .author("Carter Green")
        .about("Key-value store server")
        .arg(
            Arg::with_name("version")
                .short("V")
                .long("version")
                .help("Print the version and exit"),
        )
        .arg(
            Arg::with_name("address")
                .long("addr")
                .value_name("IP:PORT")
                .help(&format!(
                    "IP address either v4 or v6 and a port. Defaults to {}",
                    default_addr
                )),
        )
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .value_name("ENGINE")
                .help("Key-value store engine to use: either kvs or sled. Defaults to kvs"),
        )
        .get_matches();
    if args.is_present("version") {
        println!("kvs-server version {}", version);
    } else {
        let addr = args.value_of("address").unwrap_or(default_addr);
        let engine = match args.value_of("engine") {
            Some("kvs") | None => Ok(EngineImpl::Kvs),
            Some("sled") => Ok(EngineImpl::Sled),
            Some(other) => Err(format!("Invalid engine option {}", other)),
        }?;
        info!(
            log, "Starting server";
            "engine" => engine,
            "version" => version,
            "address" => addr
        );
        let cwd = std::env::current_dir()?;
        match engine {
            EngineImpl::Kvs => KvsServer::new(KvStore::open(cwd)?, &log).serve(addr)?,
            EngineImpl::Sled => KvsServer::new(SledEngine::open(cwd)?, &log).serve(addr)?,
        };
    }
    Ok(())
}
