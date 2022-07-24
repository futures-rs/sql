use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use super::*;

fn create_test_dir() -> PathBuf {
    let path: PathBuf = ".test".into();

    if path.exists() {
        remove_dir_all(path.to_owned()).unwrap();
    }

    create_dir_all(path.to_owned()).unwrap();

    path
}

#[async_std::test]
async fn test_driver_open() {
    _ = pretty_env_logger::try_init();

    let path = create_test_dir().join("./test.db");
    let mut driver = Sqlite3Driver {};

    driver.open(path.to_str().unwrap()).await.unwrap();
}
