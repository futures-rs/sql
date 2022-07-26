use super::Prepare;
use crate::waker;
use anyhow::*;

pub type Rollback = waker::WakableFuture<Result<()>>;
pub type Commit = waker::WakableFuture<Result<()>>;

pub trait Transaction: Send {
    fn prepare(&mut self, query: &str) -> Prepare;

    fn commit(&mut self) -> Commit;

    fn rollback(&mut self) -> Rollback;
}
