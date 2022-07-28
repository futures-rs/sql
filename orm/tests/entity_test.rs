use serde::{Deserialize, Serialize};

use rdbc_serde::*;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default)]
struct TestTable {
    data: Data,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default)]
struct Data {
    id: i32,
}

#[test]
fn test_serializer() {
    _ = pretty_env_logger::try_init();

    let table = TestTable::default();

    let mut serializer = Serializer {};

    table.serialize(&mut serializer).unwrap();
}

#[test]
fn test_deserializer() {
    _ = pretty_env_logger::try_init();

    let data = "";

    let mut deserializer = Deserializer { input: data };

    TestTable::deserialize(&mut deserializer).unwrap();
}
