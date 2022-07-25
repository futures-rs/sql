pub mod error;

use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
    ptr::null_mut,
    slice::from_raw_parts,
};

use error::{db_native_error, error_with_sql, Sqlite3Error};
use rdbc::{
    driver::{Connection, Driver},
    *,
};
use sqlite3_sys::*;

pub struct Sqlite3Driver {}

impl Driver for Sqlite3Driver {
    /// Open sqlite3 database with filename, which can be interpreted as a URI.
    /// See [`sqlite`](https://www.sqlite.org/c3ref/open.html) for details.
    fn open(&mut self, name: &str) -> driver::Connector {
        unsafe {
            assert!(
                sqlite3_threadsafe() != 0,
                "Sqlite3 must be compiled in thread safe mode."
            );
        }

        let (fut, waker) = driver::Connector::new();

        let mut db = std::ptr::null_mut();

        let flags =
            SQLITE_OPEN_URI | SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_NOMUTEX;

        log::debug!("open sqlite3 database: {} {:X}", name, flags);

        let c_name = str_to_cstring(&waker, name);

        if c_name.is_none() {
            return fut;
        }

        unsafe {
            let r = sqlite3_open_v2(c_name.ptr, &mut db, flags, std::ptr::null());

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

        let rc = unsafe {
            let c_str = CString::new("BEGIN").unwrap();

            sqlite3_exec(
                self.db,
                c_str.as_ptr(),
                None,
                null_mut::<c_void>(),
                null_mut::<*mut i8>(),
            )
        };

        if rc != SQLITE_OK {
            waker
                .lock()
                .unwrap()
                .ready(Err(db_native_error(self.db, rc)));

            return fut;
        }

        waker.lock().unwrap().ready(Ok(Box::new(Sqlite3Transaction {
            conn: Sqlite3Connection {
                db: self.db,
                id: self.id.clone(),
            },
            finished: false,
        })));

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

        let sqlite3_query = str_to_cstring(&waker, query);

        if sqlite3_query.is_none() {
            return fut;
        }

        let mut stmt = null_mut();

        let rc = unsafe {
            sqlite3_prepare_v2(
                self.db,
                sqlite3_query.ptr,
                sqlite3_query.len,
                &mut stmt,
                null_mut::<*const c_char>(),
            )
        };

        if rc != SQLITE_OK {
            waker
                .lock()
                .unwrap()
                .ready(Err(error_with_sql(self.db, rc, query)));
            return fut;
        }

        // If the input text contains no SQL (if the input is an empty string or a comment) then *ppStmt is set to NULL.
        if stmt.is_null() {
            waker
                .lock()
                .unwrap()
                .ready(Err(anyhow::anyhow!("invalid input sql {}", query)));
            return fut;
        }

        waker
            .lock()
            .unwrap()
            .ready(Ok(Box::new(Sqlite3Statement::new(self.db, stmt))));

        fut
    }
}

struct Sqlite3Transaction {
    conn: Sqlite3Connection,
    finished: bool,
}

impl Sqlite3Transaction {
    fn _rollback(&self) -> anyhow::Result<()> {
        let rc = unsafe {
            let c_str = CString::new("ROLLBACK").unwrap();

            sqlite3_exec(
                self.conn.db,
                c_str.as_ptr(),
                None,
                null_mut::<c_void>(),
                null_mut::<*mut i8>(),
            )
        };

        if rc != SQLITE_OK {
            return Err(error_with_sql(self.conn.db, rc, "ROLLBACK"));
        }

        Ok(())
    }
}

impl driver::Transaction for Sqlite3Transaction {
    fn commit(&mut self) -> driver::Commit {
        let (fut, waker) = driver::Commit::new();

        let rc = unsafe {
            let c_str = CString::new("COMMIT").unwrap();

            sqlite3_exec(
                self.conn.db,
                c_str.as_ptr(),
                None,
                null_mut::<c_void>(),
                null_mut::<*mut i8>(),
            )
        };

        self.finished = true;

        if rc != SQLITE_OK {
            waker
                .lock()
                .unwrap()
                .ready(Err(error_with_sql(self.conn.db, rc, "COMMIT")));
        } else {
            waker.lock().unwrap().ready(Ok(()));
        }

        fut
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        self.conn.prepare(query)
    }

    fn rollback(&mut self) -> driver::Rollback {
        let (fut, waker) = driver::Rollback::new();

        if let Err(err) = self._rollback() {
            waker.lock().unwrap().ready(Err(err));
        } else {
            waker.lock().unwrap().ready(Ok(()));
        }

        self.finished = true;

        fut
    }
}

impl Drop for Sqlite3Transaction {
    fn drop(&mut self) {
        // default to rollback all stmt .
        if !self.finished {
            _ = self._rollback();
            self.finished = true;
        }

        self.conn.db = null_mut();
    }
}

fn colunm_decltype(stmt: *mut sqlite3_stmt, i: i32) -> (driver::ColumnType, String, Option<u64>) {
    let decltype = unsafe { CStr::from_ptr(sqlite3_column_decltype(stmt, i)) }.to_string_lossy();

    match decltype.as_ref() {
        "INT" | "INTEGER" | "TINYINT" | "SMALLINT" | "MEDIUMINT" | "BIGINT"
        | "UNSIGNED BIG INT" | "INT2" | "INT8" => {
            (driver::ColumnType::I64, decltype.to_string(), Some(8))
        }
        "CHARACTER(20)"
        | "VARCHAR(255)"
        | "VARYING CHARACTER(255)"
        | "NCHAR(55)"
        | "NATIVE CHARACTER(70)"
        | "NVARCHAR(100)"
        | "TEXT"
        | "CLOB" => (driver::ColumnType::String, decltype.to_string(), None),
        "BLOB" => (driver::ColumnType::Bytes, decltype.to_string(), None),
        "REAL" | "DOUBLE" | "DOUBLE PRECISION" | "FLOAT" => {
            (driver::ColumnType::F64, decltype.to_string(), Some(8))
        }
        _ => (driver::ColumnType::String, decltype.to_string(), None),
    }
}

fn stmt_sql(stmt: *mut sqlite3_stmt) -> String {
    unsafe {
        CStr::from_ptr(sqlite3_expanded_sql(stmt))
            .to_string_lossy()
            .to_owned()
            .to_string()
    }
}

struct Sqlite3Statement {
    db: *mut sqlite3,
    stmt: *mut sqlite3_stmt,
}

impl Sqlite3Statement {
    fn new(db: *mut sqlite3, stmt: *mut sqlite3_stmt) -> Self {
        Self { db, stmt }
    }

    unsafe fn bind_args(&mut self, args: Vec<NamedValue>) -> anyhow::Result<()> {
        sqlite3_clear_bindings(self.stmt);

        for arg in args {
            let rc = match arg.value {
                driver::Value::Bytes(bytes) => {
                    let ptr = bytes.as_ptr();
                    let len = bytes.len();
                    sqlite3_bind_blob(
                        self.stmt,
                        arg.ordinal as i32,
                        ptr as *const c_void,
                        len as i32,
                        Some(std::mem::transmute(SQLITE_TRANSIENT as usize)),
                    )
                }
                driver::Value::F64(f64) => sqlite3_bind_double(self.stmt, arg.ordinal as i32, f64),

                driver::Value::I64(i64) => sqlite3_bind_int64(self.stmt, arg.ordinal as i32, i64),

                driver::Value::String(str) => {
                    let str = CString::new(str)?;

                    let ptr = str.as_ptr();
                    let len = str.as_bytes().len() as i32;

                    sqlite3_bind_text(
                        self.stmt,
                        arg.ordinal as i32,
                        ptr,
                        len,
                        Some(std::mem::transmute(SQLITE_TRANSIENT as usize)),
                    )
                }

                driver::Value::Null => SQLITE_OK,
            };

            if rc != SQLITE_OK {
                return Err(db_native_error(self.db, rc));
            }
        }

        Ok(())
    }
}

impl driver::Statement for Sqlite3Statement {
    fn execute(&mut self, args: Vec<NamedValue>) -> driver::Execute {
        let (fut, waker) = driver::Execute::new();

        if let Err(err) = unsafe { self.bind_args(args) } {
            waker.lock().unwrap().ready(Err(err));
            return fut;
        }

        log::trace!("execute sql {}", stmt_sql(self.stmt));

        let rc = unsafe { sqlite3_step(self.stmt) };

        // unsafe { sqlite3_reset(self.stmt) };

        match rc {
            SQLITE_DONE => {
                let last_insert_id = unsafe { sqlite3_last_insert_rowid(self.db) } as u64;
                let raws_affected = unsafe { sqlite3_changes(self.db) } as u64;

                waker.lock().unwrap().ready(Ok(ExecuteResult {
                    last_insert_id,
                    raws_affected,
                }));

                return fut;
            }
            SQLITE_ROW => {
                waker
                    .lock()
                    .unwrap()
                    .ready(Err(anyhow::Error::new(Sqlite3Error::UnexpectRows)));

                return fut;
            }
            _ => {
                waker
                    .lock()
                    .unwrap()
                    .ready(Err(db_native_error(self.db, rc)));

                return fut;
            }
        };
    }

    fn num_input(&self) -> Option<u32> {
        Some(unsafe { sqlite3_bind_parameter_count(self.stmt) } as u32)
    }

    fn query(&mut self, args: Vec<NamedValue>) -> driver::Query {
        let (fut, waker) = driver::Query::new();

        if let Err(err) = unsafe { self.bind_args(args) } {
            waker.lock().unwrap().ready(Err(err));
            return fut;
        }

        waker.lock().unwrap().ready(Ok(Box::new(Sqlite3Rows {
            db: self.db,
            stmt: self.stmt,
            columns: None,
            has_next: false,
        })));

        fut
    }
}

impl Drop for Sqlite3Statement {
    fn drop(&mut self) {
        if !self.stmt.is_null() {
            unsafe { sqlite3_finalize(self.stmt) };
            self.stmt = null_mut();
        }
    }
}

#[allow(dead_code)]
struct Sqlite3Rows {
    db: *mut sqlite3,
    stmt: *mut sqlite3_stmt,
    columns: Option<Vec<driver::ColumnMetaData>>,
    has_next: bool,
}

impl Sqlite3Rows {}

impl driver::Rows for Sqlite3Rows {
    fn colunms(&mut self) -> driver::Columns {
        let (fut, waker) = driver::Columns::new();

        if self.columns.is_none() {
            let mut columns = vec![];

            unsafe {
                let count = sqlite3_column_count(self.stmt);

                for i in 0..count {
                    let name = sqlite3_column_name(self.stmt, i);

                    let (_, decltype, len) = colunm_decltype(self.stmt, i);

                    columns.push(driver::ColumnMetaData {
                        column_index: i as u64,
                        column_name: CStr::from_ptr(name).to_string_lossy().to_string(),
                        column_decltype: decltype,
                        column_decltype_len: len,
                    })
                }
            };

            self.columns = Some(columns);
        }

        waker
            .lock()
            .unwrap()
            .ready(Ok(self.columns.clone().unwrap()));

        fut
    }

    fn get(&mut self, index: u64, column_type: driver::ColumnType) -> driver::RowsGet {
        log::trace!(
            "{} :get column({},{:?})",
            stmt_sql(self.stmt),
            index,
            column_type
        );

        let index = index as i32;
        let (fut, waker) = driver::RowsGet::new();

        let max_index = unsafe { sqlite3_column_count(self.stmt) };

        if index >= max_index {
            waker
                .lock()
                .unwrap()
                .ready(Err(anyhow::Error::new(Sqlite3Error::OutOfRange(
                    index as u64,
                ))));
            return fut;
        }

        if !self.has_next {
            waker
                .lock()
                .unwrap()
                .ready(Err(anyhow::Error::new(Sqlite3Error::NextDataError)));
            return fut;
        }

        let value = unsafe {
            match column_type {
                driver::ColumnType::Bytes => {
                    let len = sqlite3_column_bytes(self.stmt, index);
                    let data = sqlite3_column_blob(self.stmt, index) as *const u8;
                    let data = from_raw_parts(data, len as usize).to_owned();

                    driver::Value::Bytes(data)
                }
                driver::ColumnType::I64 => {
                    driver::Value::I64(sqlite3_column_int64(self.stmt, index))
                }
                driver::ColumnType::F64 => {
                    driver::Value::F64(sqlite3_column_double(self.stmt, index))
                }
                driver::ColumnType::String => {
                    let data = sqlite3_column_text(self.stmt, index) as *const i8;

                    driver::Value::String(CStr::from_ptr(data).to_string_lossy().to_string())
                }
                driver::ColumnType::Null => driver::Value::Null,
            }
        };

        waker.lock().unwrap().ready(Ok(value));

        fut
    }

    fn next(&mut self) -> driver::RowsNext {
        let (fut, waker) = driver::RowsNext::new();

        match unsafe { sqlite3_step(self.stmt) } {
            SQLITE_DONE => {
                self.has_next = false;
                waker.lock().unwrap().ready(Ok(false));
            }

            SQLITE_ROW => {
                self.has_next = true;
                waker.lock().unwrap().ready(Ok(true));
            }

            rc => {
                self.has_next = false;
                waker
                    .lock()
                    .unwrap()
                    .ready(Err(db_native_error(self.db, rc)));
            }
        }

        fut
    }
}

impl Drop for Sqlite3Rows {
    fn drop(&mut self) {
        unsafe { sqlite3_reset(self.stmt) };
    }
}

struct SqliteString {
    c_str: Option<CString>,
    ptr: *const c_char,
    len: i32,
}

impl SqliteString {
    fn is_none(&self) -> bool {
        self.c_str.is_none()
    }
}

fn str_to_cstring<Output>(waker: &SharedWaker<anyhow::Result<Output>>, src: &str) -> SqliteString {
    match CString::new(src) {
        Ok(c_str) => {
            let ptr = c_str.as_ptr();
            let len = c_str.as_bytes().len() as i32;
            SqliteString {
                c_str: Some(c_str),
                ptr,
                len,
            }
        }
        Err(err) => {
            waker
                .lock()
                .unwrap()
                .ready(Err(anyhow::anyhow!("parse url error: {}", err)));

            SqliteString {
                c_str: None,
                ptr: std::ptr::null(),
                len: 0,
            }
        }
    }
}

pub fn register_sqlite3() -> anyhow::Result<()> {
    rdbc::register_driver("sqlite3", Sqlite3Driver {})
}

#[cfg(test)]
mod tests;
