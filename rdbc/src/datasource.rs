use super::driver;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::database::*;

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
