pub mod driver;
mod waker;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use waker::*;

pub use driver::{ColumnMetaData, ExecuteResult, NamedValue, Value};

pub struct DataSource {
    drivers: Arc<Mutex<HashMap<String, Box<dyn driver::Driver>>>>,
}

use anyhow::*;

impl DataSource {
    pub fn new() -> Self {
        DataSource {
            drivers: Default::default(),
        }
    }

    pub fn register_driver(
        &mut self,
        name: &str,
        driver: impl driver::Driver + 'static,
    ) -> Result<()> {
        let mut drivers = self.drivers.lock().unwrap();

        if drivers.contains_key(name) {
            return Err(anyhow!("register driver {} twice", name));
        }

        drivers.insert(name.to_owned(), Box::new(driver));

        Ok(())
    }

    pub fn unregister_driver(&mut self, name: &str) -> Result<()> {
        self.drivers.lock().unwrap().remove(name);

        Ok(())
    }

    pub fn open(&mut self, name: &str, url: &str) -> Result<Database> {
        if !self.drivers.lock().unwrap().contains_key(name) {
            return Err(anyhow!("driver {} not found", name));
        }

        Ok(Database::new(name, url, self.drivers.clone()))
    }
}

pub struct Database {
    name: String,
    url: String,
    drivers: Arc<Mutex<HashMap<String, Box<dyn driver::Driver>>>>,
    connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
}

impl Database {
    fn new(
        name: &str,
        url: &str,
        drivers: Arc<Mutex<HashMap<String, Box<dyn driver::Driver>>>>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            url: url.to_owned(),
            drivers,
            connection_pool: Default::default(),
        }
    }

    async fn select_one_connection(&mut self) -> Result<Box<dyn driver::Connection>> {
        let mut connection = {
            let mut connections = self.connection_pool.lock().unwrap();

            let mut id: Option<String> = None;

            for (k, v) in connections.iter() {
                if v.is_valid() {
                    id = Some(k.to_owned());
                    break;
                }
            }

            match id {
                Some(id) => connections.remove(&id),
                _ => None,
            }
        };

        if connection.is_none() {
            let mut drivers = self.drivers.lock().unwrap();

            if let Some(driver) = drivers.get_mut(&self.name) {
                connection = Some(driver.open(&self.url).await?);
            }
        }

        Ok(connection.unwrap())
    }

    /// Prepare creates a prepared statement for later queries or executions.
    pub async fn prepare(&mut self, query: &str) -> Result<Statement> {
        let mut connection = self.select_one_connection().await?;

        let statement = connection.prepare(query).await?;

        Ok(Statement::new(
            self.connection_pool.clone(),
            Some(connection),
            statement,
        ))
    }

    /// Starts and returns a new transaction.
    pub async fn begin(&mut self) -> Result<Transaction> {
        let mut connection = self.select_one_connection().await?;

        let tx = connection.begin().await?;

        Ok(Transaction::new(
            self.connection_pool.clone(),
            Some(connection),
            tx,
        ))
    }
}

/// The [`driver::Transaction`] wrapper
pub struct Transaction {
    inner: Box<dyn driver::Transaction>,
    connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
    conn: Option<Box<dyn driver::Connection>>,
}

impl Transaction {
    pub(crate) fn new(
        connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
        conn: Option<Box<dyn driver::Connection>>,
        inner: Box<dyn driver::Transaction>,
    ) -> Self {
        Self {
            inner,
            connection_pool,
            conn,
        }
    }

    pub async fn prepare(&mut self, query: &str) -> Result<Statement> {
        let statement = self.inner.prepare(query).await?;

        Ok(Statement::new(
            self.connection_pool.clone(),
            None,
            statement,
        ))
    }

    pub async fn commit(&mut self) -> Result<()> {
        self.inner.commit().await
    }

    pub async fn rollback(&mut self) -> Result<()> {
        self.inner.rollback().await
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.connection_pool
                .lock()
                .unwrap()
                .insert(conn.id().to_owned(), conn);
        }
    }
}

/// The [`driver::Statement`] wrapper
pub struct Statement {
    conn: Option<Box<dyn driver::Connection>>,
    statement: Box<dyn driver::Statement>,
    connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
}

impl Statement {
    pub(crate) fn new(
        connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
        conn: Option<Box<dyn driver::Connection>>,
        statement: Box<dyn driver::Statement>,
    ) -> Self {
        Statement {
            connection_pool,
            conn,
            statement,
        }
    }

    pub fn num_input(&self) -> Option<u32> {
        self.statement.num_input()
    }

    /// Executes a query that doesn't return rows, such
    /// as an INSERT or UPDATE.
    pub async fn execute(&mut self, args: Vec<NamedValue>) -> Result<ExecuteResult> {
        self.statement.execute(args).await
    }

    /// executes a query that may return rows, such as a
    /// SELECT.
    pub async fn query(&mut self, args: Vec<NamedValue>) -> Result<Rows> {
        let rows = self.statement.query(args).await?;
        Ok(Rows::new(rows))
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.connection_pool
                .lock()
                .unwrap()
                .insert(conn.id().to_owned(), conn);
        }
    }
}

/// [`driver::Rows`] wrapper
pub struct Rows {
    inner: Box<dyn driver::Rows>,
}

impl Rows {
    pub(crate) fn new(inner: Box<dyn driver::Rows>) -> Self {
        Self { inner }
    }

    pub async fn colunms(&mut self) -> Result<Vec<ColumnMetaData>> {
        self.inner.colunms().await
    }

    pub async fn next(&mut self) -> Result<bool> {
        self.inner.next().await
    }

    pub async fn get(&mut self, index: u64) -> Result<Option<Value>> {
        self.inner.get(index).await
    }
}

#[cfg(feature = "global-datasource")]
mod global {
    use super::*;

    fn global_datasource() -> &'static mut DataSource {
        static mut CONF: std::mem::MaybeUninit<DataSource> = std::mem::MaybeUninit::uninit();
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| unsafe {
            CONF.as_mut_ptr().write(DataSource::new());
        });
        unsafe { &mut *CONF.as_mut_ptr() }
    }

    pub fn register_driver(name: &str, driver: impl driver::Driver + 'static) -> Result<()> {
        global_datasource().register_driver(name, driver)
    }

    pub fn unregister_driver(name: &str) -> Result<()> {
        global_datasource().unregister_driver(name)
    }

    pub fn open(name: &str, url: &str) -> Result<Database> {
        global_datasource().open(name, url)
    }
}

#[cfg(feature = "global-datasource")]
pub use global::*;
