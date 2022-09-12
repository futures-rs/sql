mod database;
mod datasource;
pub mod driver;
mod prepare;
mod rows;
mod statement;
mod transaction;

pub use database::*;
pub use datasource::*;
pub use prepare::*;
pub use rows::*;
pub use statement::*;
pub use transaction::*;

pub use driver::{Arg, ColumnMetaData, ColumnType, ExecuteResult, Placeholder, Value};

pub use futures_any;
pub use futures_signal;

#[cfg(feature = "global-datasource")]
mod global {
    use super::*;
    use anyhow::*;

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
