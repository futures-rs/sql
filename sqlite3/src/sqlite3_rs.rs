/// ! sqlite3 c api wrapper mod
///
use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
    ptr::null_mut,
    slice::from_raw_parts,
};

use super::error;

use sqlite3_sys::*;

use anyhow::Result;

use rdbc::driver;

pub fn colunm_decltype(
    stmt: *mut sqlite3_stmt,
    i: i32,
) -> (driver::ColumnType, String, Option<u64>) {
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

pub fn stmt_sql(stmt: *mut sqlite3_stmt) -> String {
    unsafe {
        CStr::from_ptr(sqlite3_expanded_sql(stmt))
            .to_string_lossy()
            .to_owned()
            .to_string()
    }
}

/// sqlite connection object
pub struct Connection {
    db: *mut sqlite3,
    pub id: String,
}

unsafe impl Send for Connection {}

impl Connection {
    pub fn open(name: &str) -> Result<Self> {
        unsafe {
            assert!(
                sqlite3_threadsafe() != 0,
                "Sqlite3 must be compiled in thread safe mode."
            );
        }

        let mut db = std::ptr::null_mut();

        let flags =
            SQLITE_OPEN_URI | SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_NOMUTEX;

        log::debug!("open sqlite3 database: {} {:X}", name, flags);

        let c_name = CString::new(name)?;

        unsafe {
            let r = sqlite3_open_v2(c_name.as_ptr(), &mut db, flags, std::ptr::null());

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

                return Err(e);
            } else {
                return Ok(Self {
                    db,
                    id: format!("{:?}", db),
                });
            }
        }
    }

    pub fn begin(&mut self) -> Result<Transaction> {
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
            return Err(error::db_native_error(self.db, rc));
        }

        Ok(Transaction {
            conn: Connection {
                db: self.db,
                id: self.id.clone(),
            },
            finished: false,
            id: uuid::Uuid::new_v4().to_string(), // Use the randomly generated uuid as tx id
        })
    }

    pub fn prepare(&mut self, query: &str) -> Result<Statement> {
        let sqlite3_query = CString::new(query)?;

        let mut stmt = null_mut();

        let rc = unsafe {
            sqlite3_prepare_v2(
                self.db,
                sqlite3_query.as_ptr(),
                sqlite3_query.as_bytes().len() as i32,
                &mut stmt,
                null_mut::<*const c_char>(),
            )
        };

        if rc != SQLITE_OK {
            return Err(error::error_with_sql(self.db, rc, query));
        }

        // If the input text contains no SQL (if the input is an empty string or a comment) then *ppStmt is set to NULL.
        if stmt.is_null() {
            return Err(anyhow::anyhow!("invalid input sql {}", query));
        }

        Ok(Statement {
            db: self.db,
            stmt,
            id: format!("{:?}", stmt),
        })
    }
}

impl Drop for Connection {
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

pub struct Statement {
    db: *mut sqlite3,
    stmt: *mut sqlite3_stmt,
    pub id: String,
}

impl Statement {
    unsafe fn bind_args(&mut self, args: Vec<rdbc::NamedValue>) -> anyhow::Result<()> {
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
                return Err(error::db_native_error(self.db, rc));
            }
        }

        Ok(())
    }

    pub fn execute(&mut self, args: Vec<rdbc::NamedValue>) -> Result<driver::ExecuteResult> {
        unsafe { self.bind_args(args) }?;

        log::trace!("execute sql {}", stmt_sql(self.stmt));

        let rc = unsafe { sqlite3_step(self.stmt) };

        // unsafe { sqlite3_reset(self.stmt) };

        match rc {
            SQLITE_DONE => {
                let last_insert_id = unsafe { sqlite3_last_insert_rowid(self.db) } as u64;
                let raws_affected = unsafe { sqlite3_changes(self.db) } as u64;

                return Ok(driver::ExecuteResult {
                    last_insert_id,
                    raws_affected,
                });
            }
            SQLITE_ROW => {
                return Err(anyhow::Error::new(error::Sqlite3Error::UnexpectRows));
            }
            _ => {
                return Err(error::db_native_error(self.db, rc));
            }
        };
    }

    pub fn num_input(&self) -> Option<u32> {
        Some(unsafe { sqlite3_bind_parameter_count(self.stmt) } as u32)
    }

    pub fn query(&mut self, args: Vec<rdbc::NamedValue>) -> Result<Rows> {
        unsafe { self.bind_args(args) }?;

        return Ok(Rows {
            db: self.db,
            stmt: self.stmt,
            columns: None,
            has_next: false,
            id: uuid::Uuid::new_v4().to_string(),
        });
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        if !self.stmt.is_null() {
            unsafe { sqlite3_finalize(self.stmt) };
            self.stmt = null_mut();
        }
    }
}

pub struct Transaction {
    conn: Connection,
    finished: bool,
    pub id: String,
}

impl Transaction {
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
            return Err(error::error_with_sql(self.conn.db, rc, "ROLLBACK"));
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
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
            Err(error::error_with_sql(self.conn.db, rc, "COMMIT"))
        } else {
            Ok(())
        }
    }

    pub fn prepare(&mut self, query: &str) -> Result<Statement> {
        self.conn.prepare(query)
    }

    pub fn rollback(&mut self) -> Result<()> {
        self.finished = true;

        self._rollback()
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // default to rollback all stmt .
        if !self.finished {
            _ = self._rollback();
            self.finished = true;
        }

        self.conn.db = null_mut();
    }
}

pub struct Rows {
    db: *mut sqlite3,
    stmt: *mut sqlite3_stmt,
    columns: Option<Vec<driver::ColumnMetaData>>,
    has_next: bool,
    pub id: String,
}

impl Rows {
    pub fn colunms(&mut self) -> Result<Vec<driver::ColumnMetaData>> {
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

        Ok(self.columns.clone().unwrap())
    }

    pub fn get(&mut self, index: u64, column_type: driver::ColumnType) -> Result<rdbc::Value> {
        log::trace!(
            "{} :get column({},{:?})",
            stmt_sql(self.stmt),
            index,
            column_type
        );

        let index = index as i32;

        let max_index = unsafe { sqlite3_column_count(self.stmt) };

        if index >= max_index {
            return Err(anyhow::Error::new(error::Sqlite3Error::OutOfRange(
                index as u64,
            )));
        }

        if !self.has_next {
            return Err(anyhow::Error::new(error::Sqlite3Error::NextDataError));
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

        Ok(value)
    }

    pub fn next(&mut self) -> Result<bool> {
        match unsafe { sqlite3_step(self.stmt) } {
            SQLITE_DONE => {
                self.has_next = false;
                Ok(false)
            }

            SQLITE_ROW => {
                self.has_next = true;
                Ok(true)
            }

            rc => {
                self.has_next = false;
                Err(error::db_native_error(self.db, rc))
            }
        }
    }
}

impl Drop for Rows {
    fn drop(&mut self) {
        unsafe { sqlite3_reset(self.stmt) };
    }
}
