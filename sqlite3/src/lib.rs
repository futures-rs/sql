pub mod error;

pub mod sqlite3_rs;

// #[cfg(not(feature = "async-sqlite3"))]
pub mod sync_driver;

#[cfg(feature = "async-sqlite3")]
pub mod async_driver;

// #[cfg_attr(feature = "async-sqlite3", path = "./sync_driver.rs")]
// pub mod driver;

pub fn register_sqlite3() -> anyhow::Result<()> {
    #[cfg(feature = "async-sqlite3")]
    return rdbc::register_driver("sqlite3", async_driver::AsyncDriver::new());

    #[cfg(not(feature = "async-sqlite3"))]
    return rdbc::register_driver("sqlite3", sync_driver::SyncDriver {});
}

#[cfg(test)]
mod tests;
