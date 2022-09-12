use anyhow::*;
use futures_signal::Signal;

pub type Connector = Signal<Result<Box<dyn super::Connection>>>;

pub trait Driver: Send {
    /// Open returns new connection to the database
    fn open(&mut self, name: &str) -> Connector;
}
