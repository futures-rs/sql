use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::driver;
use super::driver::{ExecuteResult, NamedValue};
use super::rows::*;
use anyhow::*;

/// The [`driver::Statement`] wrapper
pub struct Statement {
    conn: Option<Box<dyn driver::Connection>>,
    statement: Box<dyn driver::Statement>,
    connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
}

unsafe impl Send for Statement {}

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
        let fut = self.statement.execute(args);

        // drop(self);

        fut.await
    }

    /// executes a query that may return rows, such as a
    /// SELECT.
    pub async fn query(&mut self, args: Vec<NamedValue>) -> Result<Rows> {
        let fut = self.statement.query(args);

        let rows = fut.await?;

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
