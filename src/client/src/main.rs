use kvs_client::Client;

use clap::{App, AppSettings, Arg, SubCommand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let ip_port_arg = Arg::with_name("address")
        .long("addr")
        .value_name("IP:PORT")
        .help("IP address either v4 or v6 and a port of the server. Defaults to localhost:4000");
    let args = App::new("kvs-client")
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
                    Arg::with_name("key")
                        .value_name("KEY")
                        .help("Key where to store the value")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("value")
                        .value_name("VALUE")
                        .help("Value to store under key")
                        .required(true)
                        .index(2),
                )
                .arg(&ip_port_arg),
        )
        .subcommand(
            SubCommand::with_name("get")
                .help("Get the value of a key")
                .arg(
                    Arg::with_name("key")
                        .value_name("KEY")
                        .help("Key whose value will be retrieved")
                        .required(true)
                        .index(1),
                )
                .arg(&ip_port_arg),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .help("Remove a key and its value")
                .arg(
                    Arg::with_name("key")
                        .value_name("KEY")
                        .help("Key to remove")
                        .required(true)
                        .index(1),
                )
                .arg(&ip_port_arg),
        )
        .get_matches();
    if args.is_present("version") {
        println!("kvs-client version {}", env!("CARGO_PKG_VERSION"));
    } else {
        let default_addr = "127.0.0.1:4000";

        match args.subcommand() {
            ("set", Some(sub)) => {
                let mut client = Client::connect(sub.value_of("address").unwrap_or(default_addr))?;
                client.set(
                    // Safe to unwrap because arguments are required
                    sub.value_of("key").unwrap().to_owned(),
                    sub.value_of("value").unwrap().to_owned(),
                )?;
            }
            ("get", Some(sub)) => {
                let mut client = Client::connect(sub.value_of("address").unwrap_or(default_addr))?;
                let res = client.get(sub.value_of("key").unwrap().to_owned())?;
                match res {
                    Some(value) => println!("{}", value),
                    None => println!("Key not found"),
                };
            }
            ("rm", Some(sub)) => {
                let mut client = Client::connect(sub.value_of("address").unwrap_or(default_addr))?;
                let res = client.remove(sub.value_of("key").unwrap().to_owned());
                if let Err(kvs::Error::KeyNotFound { .. }) = res {
                    println!("Key not found");
                    std::process::exit(1);
                }
                if let Err(kvs::Error::Server { msg }) = res.as_ref() {
                    if msg.starts_with("Key not found") {
                        println!("Key not found");
                        std::process::exit(1);
                    }
                }
                res?;
            }
            _ => panic!("Unexpected subcommand"),
        }
    }
    Ok(())
}
