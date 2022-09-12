use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use criterion::async_executor::FuturesExecutor;

use std::time::Duration;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use rdbc_sqlite3::*;

use rdbc::{Placeholder, Preparable, Value};

#[allow(dead_code)]
async fn prepare_benchmark() -> rdbc::Database {
    _ = register_sqlite3();

    let path: PathBuf = ".test".into();

    if path.exists() {
        remove_dir_all(path.to_owned()).unwrap();
    }

    create_dir_all(path.to_owned()).unwrap();

    let path = path.join("test.db");

    let path = format!("file:{}", path.to_string_lossy());

    let mut db = rdbc::open("sqlite3", &path).unwrap();

    // let mut db = rdbc::open("sqlite3", "file:memdb_commit?mode=memory&cache=shared").unwrap();

    let mut stmt = db
        .prepare("CREATE TABLE t(x INTEGER PRIMARY KEY ASC, y TEXT, z NUMERIC);")
        .await
        .unwrap();

    stmt.execute(vec![]).await.unwrap();

    db
}

async fn insert_one_row(mut db: rdbc::Database) {
    let mut stmt = db.prepare("INSERT INTO t(y,z) VALUES(?,?);").await.unwrap();

    _ = stmt
        .execute(vec![
            rdbc::Arg {
                pos: Placeholder::Index(1),
                value: Value::String("hello world".to_owned()),
            },
            rdbc::Arg {
                pos: Placeholder::Index(2),
                value: Value::String("7.82910138827292".to_owned()),
            },
        ])
        .await
        .unwrap();
}

fn insert_benchmark(c: &mut Criterion) {
    let db = async_std::task::block_on(async { prepare_benchmark().await });

    let mut group = c.benchmark_group("example-cuad");

    group.measurement_time(Duration::from_secs(10));

    group.bench_function("insert benchmark", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| insert_one_row(db.clone()));
    });

    group.finish();
}

criterion_group!(benches, insert_benchmark);

criterion_main!(benches);
