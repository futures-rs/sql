use super::driver;
use super::driver::*;
use anyhow::*;

/// [`driver::Rows`] wrapper
pub struct Rows {
    inner: Box<dyn driver::Rows>,
}

impl Rows {
    pub(crate) fn new(inner: Box<dyn driver::Rows>) -> Self {
        Self { inner }
    }

    pub async fn colunms(&mut self) -> Result<Vec<ColumnMetaData>> {
        self.inner.colunms().await
    }

    pub async fn next(&mut self) -> Result<bool> {
        self.inner.next().await
    }

    pub async fn get(&mut self, index: u64, column_type: driver::ColumnType) -> Result<Value> {
        self.inner.get(index, column_type).await
    }
}
