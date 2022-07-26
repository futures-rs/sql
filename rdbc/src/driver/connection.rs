use super::Statement;
use super::Transaction;
use crate::waker;
use anyhow::*;

pub type Prepare = waker::WakableFuture<Result<Box<dyn Statement>>>;
pub type Begin = waker::WakableFuture<Result<Box<dyn Transaction>>>;

pub trait Connection: Send {
    /// Returns a prepared statement, bound to this connection.
    fn prepare(&mut self, query: &str) -> Prepare;

    fn begin(&mut self) -> Begin;

    /// Driver use this function to return connection status
    fn is_valid(&self) -> bool;

    /// Get connection id
    fn id(&self) -> &str;
}
