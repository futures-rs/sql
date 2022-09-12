use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{driver::Connection, statement, Preparable};

use super::driver;
use super::statement::*;
use super::transaction::*;
use anyhow::Result;
use futures::{FutureExt, TryFutureExt};
use futures_any::prelude::AnyFutureEx;

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

impl Preparable for Database {
    fn prepare(
        &mut self,
        query: &str,
    ) -> futures_any::prelude::AnyFuture<anyhow::Result<Statement>> {
        let connection_pool = self.connection_pool.clone();

        self.select_one_connection()
            .and_then(|mut conn| async {
                let stmt = conn.prepare(query).await?;

                Ok((stmt, conn))
            })
            .map_ok(|(statement, connection)| {
                Statement::new(connection_pool, Some(connection), statement)
            })
            .to_any_future()
    }
}
