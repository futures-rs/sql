use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::*;

#[async_trait::async_trait]
/// Represents database driver that can be shared between threads, and can therefore implement
/// a connection pool
pub trait Driver: Sync + Send {
    async fn connect(&self, url: &str) -> Result<Box<dyn Connection>>;
}

#[async_trait::async_trait]
/// Database connection session
pub trait Connection: Sync + Send {
    /// Create a prepared statement for execution
    async fn prepare(&mut self, sql: &str) -> Result<Box<dyn Statement>>;

    async fn begin(&mut self) -> Result<Box<dyn Transaction>>;
}

#[async_trait::async_trait]
pub trait Transaction {
    /// Create a prepared statement for execution
    async fn prepare(&mut self, sql: &str) -> Result<Box<dyn Statement>>;

    /// rollback the transaction
    async fn rollback(&mut self) -> Result<()>;

    /// commits the transaction
    async fn commit(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
/// Represents an executable SQL object
pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    async fn execute_query(&mut self, params: &[Value]) -> Result<Box<dyn ResultSet>>;

    /// Execute a query that is expected to update some rows.
    async fn execute_update(&mut self, params: &[Value]) -> Result<u64>;
}

pub struct Value {}

#[async_trait::async_trait]
/// Result set from executing a query against a statement
pub trait ResultSet {
    /// get meta data about this result set
    async fn meta_data(&self) -> Result<Box<dyn ResultSetMetaData>>;

    /// Move the cursor to the next available row if one exists and return true if it does
    async fn next(&mut self) -> bool;

    async fn value_of(&self, i: u64) -> Result<Option<i8>>;
}

#[async_trait::async_trait]
/// Meta data for result set
pub trait ResultSetMetaData {
    async fn num_columns(&self) -> u64;
    async fn column_name(&self, i: u64) -> String;
    async fn column_type(&self, i: u64) -> DataType;
}

pub enum DataType {}

#[derive(Clone)]
pub struct DriverManager {
    drivers: Arc<Mutex<HashMap<String, Box<dyn Driver>>>>,
}

impl DriverManager {
    /// Create new driver manager
    pub fn new() -> Self {
        DriverManager {
            drivers: Default::default(),
        }
    }

    /// Register driver by provider name .
    pub fn register_driver(&mut self, name: &str, driver: impl Driver + 'static) -> Result<()> {
        let mut drivers = self.drivers.lock().unwrap();

        if drivers.contains_key(name) {
            return Err(anyhow!("exists sql driver {}", name));
        }

        drivers.insert(name.to_owned(), Box::new(driver));

        Ok(())
    }

    pub fn unregister_driver(&mut self, name: &str) -> Result<()> {
        self.drivers.lock().unwrap().remove(name);

        Ok(())
    }

    /// Open opens a database specified by its database driver name and a driver-specific data source name,
    /// usually consisting of at least a database name and connection information.
    pub async fn open(&self, name: &str, url: &str) -> Result<Box<dyn Connection>> {
        let drivers = self.drivers.lock().unwrap();

        if !drivers.contains_key(name) {
            return Err(anyhow!("unknown sql driver {}", name));
        }

        let driver = drivers.get(name).unwrap();

        driver.connect(url).await
    }
}

/// Global singleton driver manager object
fn singleton() -> &'static mut DriverManager {
    static mut CONF: std::mem::MaybeUninit<DriverManager> = std::mem::MaybeUninit::uninit();
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        CONF.as_mut_ptr().write(DriverManager::new());
    });
    unsafe { &mut *CONF.as_mut_ptr() }
}

/// Register driver by provider name .
pub fn register_driver(name: &str, driver: impl Driver + 'static) -> Result<()> {
    singleton().register_driver(name, driver)
}

/// Unregister driver
pub fn unregister_driver(name: &str) -> Result<()> {
    singleton().unregister_driver(name)
}

/// Open opens a database specified by its database driver name and a driver-specific data source name,
/// usually consisting of at least a database name and connection information.
pub async fn open(name: &str, url: &str) -> Result<Box<dyn Connection>> {
    singleton().open(name, url).await
}
