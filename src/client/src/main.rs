use kvs::{KvsEngine, KvStore};

use clap::{App, AppSettings, Arg, SubCommand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("kvs")
        .author("Carter Green")
        .about("Key-value store client")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("version")
                .short("V")
                .long("version")
                .help("Print the version and exit"),
        )
        .subcommand(
            SubCommand::with_name("set")
                .help("Set the value of a key")
                .arg(
                    Arg::with_name("KEY")
                        .help("Key where to store the value")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("Value to store under key")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .help("Get the value of a key")
                .arg(
                    Arg::with_name("KEY")
                        .help("Key whose value will be retrieved")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .help("Remove a key and its value")
                .arg(
                    Arg::with_name("KEY")
                        .help("Key to remove")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();
    if args.is_present("version") {
        println!("kvs version {}", env!("CARGO_PKG_VERSION"));
    } else {
        let cwd = std::env::current_dir()?;
        let mut store = KvStore::open(cwd)?;
        match args.subcommand() {
            ("set", Some(sub)) => {
                // Safe to unwrap because arguments are required
                store.set(
                    sub.value_of("KEY").unwrap().to_owned(),
                    sub.value_of("VALUE").unwrap().to_owned(),
                )?;
            }
            ("get", Some(sub)) => {
                let value = store.get(sub.value_of("KEY").unwrap().to_owned())?;
                match value {
                    Some(value) => println!("{}", value),
                    None => println!("Key not found"),
                };
            }
            ("rm", Some(sub)) => {
                let res = store.remove(sub.value_of("KEY").unwrap().to_owned());
                if let Err(kvs::Error::KeyNotFound { .. }) = res {
                    println!("Key not found");
                    std::process::exit(1);
                }
                res?;
            }
            _ => panic!("Unexpected subcommand"),
        }
    }
    Ok(())
}
