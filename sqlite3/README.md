# RDBC Official Sqlite3 Driver 

## async-sqlite3

This [**optional feature**](https://doc.rust-lang.org/cargo/reference/features.html) is created for architectural validation (RDBC async task system), please don't use in production environment.


## benchmark

This crate use [criterion.rs](https://bheisler.github.io/criterion.rs) for benchmarking.

To run benchmark, use the following command:

```bash
cargo bench
```

## usage

```rust
use rdbc::*;
use rdbc_sqlite3::*;

fn main() {
    register_sqlite3().unwrap();


    // Support sqlite3 uri parse
    let mut db = open("sqlite3", "file:memdb?mode=memory&cache=shared").unwrap();

    .....
}
```