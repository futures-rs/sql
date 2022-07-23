use crate::waker;
use anyhow::*;

pub type Connector = waker::WakableFuture<Result<Box<dyn super::Connection>>>;

trait Driver {
    /// Open returns new connection to the database
    fn open(name: &str) -> Connector;
}
