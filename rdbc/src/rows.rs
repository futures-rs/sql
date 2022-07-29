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

    pub async fn get<Pos>(&mut self, pos: Pos, column_type: driver::ColumnType) -> Result<Value>
    where
        Pos: Into<Placeholder>,
    {
        self.inner.get(pos.into(), column_type).await
    }
}
