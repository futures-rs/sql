use super::driver;
use super::statement::*;
use anyhow::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
