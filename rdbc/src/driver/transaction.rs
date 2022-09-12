use super::Prepare;

use anyhow::*;

use futures_signal::Signal;

pub type Rollback = Signal<Result<()>>;
pub type Commit = Signal<Result<()>>;

/// Driver transaction trait .
///
/// The driver must ensure that uncommitted transaction objects automatically perform
/// a [`Transaction::rollback`] operation when they are dropped.
pub trait Transaction: Send {
    fn prepare(&mut self, query: &str) -> Prepare;

    fn commit(&mut self) -> Commit;

    fn rollback(&mut self) -> Rollback;
}
