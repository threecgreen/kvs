use kvs::{KvStore, KvsEngine};

use clap::{App, Arg};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("kvs-server")
        .author("Carter Green")
        .about("Key-value store server")
        .arg(
            Arg::with_name("version")
                .short("V")
                .long("version")
                .help("Print the version and exit"),
        )
        .arg(Arg::with_name("address").long("addr").value_name("IP:PORT"))
        .help("IP address either v4 or v6 and a port. Defaults to localhost:4000")
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .value_name("ENGINE")
                .help("Key-value store engine to use: either kvs or sled. Defaults to kvs"),
        )
        .get_matches();
    if args.is_present("version") {
        println!("kvs-server version {}", env!("CARGO_PKG_VERSION"));
    } else {
        let addr = args.value_of("address").unwrap_or("127.0.0.1:4000");
        let engine: Box<dyn KvsEngine> = match args.value_of("engine") {
            Some("kvs") | None => Ok(Box::new(KvStore::open(std::env::current_dir()?)?)),
            Some("sled") => todo!("Implement sled integration"),
            Some(other) => Err(format!("Invalid engine option {}", other)),
        }?;
    }
    Ok(())
}
