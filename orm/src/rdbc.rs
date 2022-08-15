use crate::{Joinable, Serializable, Serializer, TableRef};

pub struct RdbcSerializer {}

impl RdbcSerializer {
    pub fn new(metadata: &'static TableRef) -> Self {
        RdbcSerializer {}
    }
}

impl Serializer for RdbcSerializer {
    fn serialize_col(&mut self, col: &crate::ColumnRef, value: rdbc::Value) -> anyhow::Result<()> {
        Ok(())
    }

    fn serialize_join_to<Join>(
        &mut self,
        col: &crate::ColumnRef,
        join: Vec<std::sync::Arc<Join>>,
    ) -> anyhow::Result<()>
    where
        Join: Joinable + Serializable,
    {
        Ok(())
    }
}
