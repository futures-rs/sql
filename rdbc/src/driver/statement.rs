use crate::waker;
use anyhow::*;

pub type Execute = waker::WakableFuture<Result<ExecuteResult>>;
pub type Query = waker::WakableFuture<Result<Box<dyn Rows>>>;
pub type Columns = waker::WakableFuture<Result<Vec<ColumnMetaData>>>;
pub type RowsNext = waker::WakableFuture<Result<bool>>;
pub type RowsGet = waker::WakableFuture<Result<Option<Value>>>;

pub trait Statement {
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
    pub name: String,
    pub ordinal: u32,
    pub value: Value,
}

pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
}

pub struct ExecuteResult {
    pub last_insert_id: u64,
    pub raws_affected: u64,
}

pub trait Rows {
    fn colunms(&mut self) -> Columns;

    fn next(&mut self) -> RowsNext;

    fn get(&mut self, index: u64) -> RowsGet;
}

pub struct ColumnMetaData {
    pub column_index: u64,
    pub column_name: String,
    pub column_type: ColumnType,
}

pub enum ColumnType {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
}
