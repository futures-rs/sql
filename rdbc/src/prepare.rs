use futures_any::prelude::AnyFuture;

use crate::Statement;

pub trait Preparable {
    fn prepare(&mut self, query: &str) -> AnyFuture<anyhow::Result<Statement>>;
}
