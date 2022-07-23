use crate::waker;
use anyhow::*;

pub type Connector = waker::WakableFuture<Result<Box<dyn super::Connection>>>;

pub trait Driver {
    /// Open returns new connection to the database
    fn open(&mut self, name: &str) -> Connector;
}
