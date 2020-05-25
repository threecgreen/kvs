# Persistent key-value store
Implemented with write-ahead log, based on the project described
[here](https://github.com/pingcap/talent-plan/tree/master/courses/rust).

## CLI Usage
The current directory is used for persisting the database.
```sh
$ cargo run -- set KEY VALUE
$ cargo run -- get KEY
VALUE
$ cargo run -- rm KEY
$ cargo run -- get KEY
Key not found
```

## Usage as a library
```rust
use kvs::KvStore;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut store = KvStore::open("/path/to/db");
    let key = "KEY";
    store.set(key, "VALUE");
    println!("The value at {} is {}", key, store.get("VALUE").unwrap());

    Ok(())
}
```
