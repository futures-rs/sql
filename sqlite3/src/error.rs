use std::os::raw::{c_char, c_int};

use sqlite3_sys::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Sqlite3Error {
    #[error("Sqlite3 native error: code({0}) {1}")]
    NativeError(i32, String),

    #[error("Sqlite3 step return unexpect rows")]
    UnexpectRows,

    #[error("Call next first or no more rows")]
    NextDataError,

    #[error("Sqlite3 get column data out of range {0}")]
    OutOfRange(u64),

    #[error("stmt '{0}' bind named arg({1}) failed")]
    BindArgError(String, String),

    #[error("Sqlite3 get column by name {0}, not found")]
    UnknownColumn(String),
}

pub fn native_error(code: i32, message: String) -> anyhow::Error {
    anyhow::Error::new(Sqlite3Error::NativeError(code, message))
}

pub fn db_native_error(db: *mut sqlite3, code: c_int) -> anyhow::Error {
    let errmsg = unsafe { errmsg_to_string(sqlite3_errmsg(db)) };

    native_error(code, errmsg)
}

pub fn error_with_sql(db: *mut sqlite3, code: c_int, sql: &str) -> anyhow::Error {
    let errmsg = unsafe { errmsg_to_string(sqlite3_errmsg(db)) };

    native_error(code, format!("{}, with SQL {}", errmsg, sql))
}

unsafe fn errmsg_to_string(errmsg: *const c_char) -> String {
    std::ffi::CStr::from_ptr(errmsg)
        .to_string_lossy()
        .into_owned()
}
