use std::os::raw::{c_char, c_int};

use sqlite3_sys::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Sqlite3Error {
    #[error("Sqlite3 native error: code({0}) {1}")]
    NativeError(i32, String),
}

pub fn native_error(code: i32, message: String) -> anyhow::Error {
    anyhow::Error::new(Sqlite3Error::NativeError(code, message))
}

pub fn db_native_error(db: *mut sqlite3, code: c_int) -> anyhow::Error {
    let errmsg = unsafe { errmsg_to_string(sqlite3_errmsg(db)) };

    native_error(code, errmsg)
}

unsafe fn errmsg_to_string(errmsg: *const c_char) -> String {
    std::ffi::CStr::from_ptr(errmsg)
        .to_string_lossy()
        .into_owned()
}
