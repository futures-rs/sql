use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::driver;
use super::statement::*;
use super::transaction::*;
use anyhow::*;

#[derive(Clone)]
pub struct Database {
    name: String,
    url: String,
    drivers: Arc<Mutex<HashMap<String, Box<dyn driver::Driver>>>>,
    connection_pool: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
}

impl Database {
    pub(crate) fn new(
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
            let fut = {
                let mut drivers = self.drivers.lock().unwrap();

                if let Some(driver) = drivers.get_mut(&self.name) {
                    Some(driver.open(&self.url))
                } else {
                    None
                }
            };

            if fut.is_none() {
                return Err(anyhow::anyhow!("driver {} not found", self.name));
            }

            connection = Some(fut.unwrap().await?);
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
