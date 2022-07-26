use crate::waker;
use anyhow::*;

pub type Execute = waker::WakableFuture<Result<ExecuteResult>>;
pub type Query = waker::WakableFuture<Result<Box<dyn Rows>>>;
pub type Columns = waker::WakableFuture<Result<Vec<ColumnMetaData>>>;
pub type RowsNext = waker::WakableFuture<Result<bool>>;
pub type RowsGet = waker::WakableFuture<Result<Value>>;

pub trait Statement: Send {
    /// Returns the number of placeholder parameters.
    ///
    /// May returns [`None`], if the driver doesn't know its number of placeholder
    fn num_input(&self) -> Option<u32>;

    /// Executes a query that doesn't return rows, such
    /// as an INSERT or UPDATE.
    fn execute(&mut self, args: Vec<NamedValue>) -> Execute;

    /// executes a query that may return rows, such as a
    /// SELECT.
    fn query(&mut self, args: Vec<NamedValue>) -> Query;
}

pub struct NamedValue {
    pub name: Option<String>,
    pub ordinal: u64,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Null,
}

pub struct ExecuteResult {
    pub last_insert_id: u64,
    pub raws_affected: u64,
}

pub trait Rows: Send {
    fn colunms(&mut self) -> Columns;

    fn next(&mut self) -> RowsNext;

    fn get(&mut self, index: u64, column_type: ColumnType) -> RowsGet;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnMetaData {
    pub column_index: u64,
    pub column_name: String,
    pub column_decltype: String,
    pub column_decltype_len: Option<u64>,
}

#[derive(Debug)]
pub enum ColumnType {
    I64,
    F64,
    String,
    Bytes,
    Null,
}
