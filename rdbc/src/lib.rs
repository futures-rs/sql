pub mod driver;
mod waker;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub use waker::*;

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
    connections: Arc<Mutex<HashMap<String, Box<dyn driver::Connection>>>>,
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
            connections: Default::default(),
        }
    }

    async fn select_one_connection(&mut self) -> Result<Box<dyn driver::Connection>> {
        // let mut connection = self.select_one_connection();

        // if connection.is_none() {
        //     let mut drivers = self.drivers.lock().unwrap();

        //     if let Some(driver) = drivers.get_mut(&self.name) {
        //         connection = Some(driver.open(&self.url).await?);
        //     }
        // }

        let mut connection = {
            let mut connections = self.connections.lock().unwrap();

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

    pub async fn prepare(&mut self, query: &str) -> Result<Box<dyn driver::Statement>> {
        let mut connection = self.select_one_connection().await?;

        connection.prepare(query).await
    }
}
