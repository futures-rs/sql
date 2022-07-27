use super::Prepare;
use crate::waker;
use anyhow::*;

pub type Rollback = waker::WakableFuture<Result<()>>;
pub type Commit = waker::WakableFuture<Result<()>>;

/// Driver transaction trait .
///
/// The driver must ensure that uncommitted transaction objects automatically perform
/// a [`Transaction::rollback`] operation when they are dropped.
pub trait Transaction: Send {
    fn prepare(&mut self, query: &str) -> Prepare;

    fn commit(&mut self) -> Commit;

    fn rollback(&mut self) -> Rollback;
}
