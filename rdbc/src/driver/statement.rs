use crate::waker;
use anyhow::*;

pub type Execute = waker::WakableFuture<Result<ExecuteResult>>;
pub type Query = waker::WakableFuture<Result<Rows>>;

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

pub struct NamedValue {}

pub struct ExecuteResult {}

pub struct Rows {}
