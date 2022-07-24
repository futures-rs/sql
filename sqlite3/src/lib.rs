pub mod error;

use rdbc::{
    driver::{Connection, Driver},
    *,
};
use sqlite3_sys::*;

pub struct Sqlite3Driver {}

impl Driver for Sqlite3Driver {
    fn open(&mut self, name: &str) -> driver::Connector {
        unsafe {
            assert!(
                sqlite3_threadsafe() != 0,
                "Sqlite3 must be compiled in thread safe mode."
            );
        }

        let (fut, waker) = driver::Connector::new();

        let mut db: *mut sqlite3 = std::ptr::null_mut();

        unsafe {
            let r = sqlite3_open(name.as_ptr() as *const std::os::raw::c_char, &mut db);

            if r != SQLITE_OK {
                let e = if db.is_null() {
                    error::native_error(r, format!("open sqlite {} failure", name))
                } else {
                    let e = error::db_native_error(db, r);

                    let r = sqlite3_close(db); // ignore result .

                    // debug output
                    if r != SQLITE_OK {
                        log::error!("close sqlite3 conn failed: code({})", r);
                    }

                    e
                };

                waker.lock().unwrap().ready(Err(e));
            } else {
                waker
                    .lock()
                    .unwrap()
                    .ready(Ok(Box::new(Sqlite3Connection::new(db))));
            }
        }

        return fut;
    }
}

struct Sqlite3Connection {
    db: *mut sqlite3,
    id: String,
}

impl Sqlite3Connection {
    fn new(db: *mut sqlite3) -> Self {
        Self {
            db,
            id: format!("{:?}", db),
        }
    }
}

impl Drop for Sqlite3Connection {
    fn drop(&mut self) {
        if !self.db.is_null() {
            log::debug!("drop db {:?}", self.db);

            let r = unsafe { sqlite3_close(self.db) }; // ignore result .

            self.db = std::ptr::null_mut(); // set db ptr to null to preventing twice drop

            // debug output
            if r != SQLITE_OK {
                log::error!("close sqlite3 conn failed: code({})", r);
            }
        }
    }
}

impl Connection for Sqlite3Connection {
    fn begin(&mut self) -> driver::Begin {
        let (fut, waker) = driver::Begin::new();

        fut
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = driver::Prepare::new();

        fut
    }
}

#[cfg(test)]
mod tests;
