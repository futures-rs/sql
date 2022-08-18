use crate::{Joinable, Serializable, Serializer};

pub struct RdbcSerializer {
    cols: Vec<rdbc::Arg>,
}

impl RdbcSerializer {
    pub fn new() -> Self {
        RdbcSerializer {
            cols: Default::default(),
        }
    }
}

impl Serializer for RdbcSerializer {
    fn serialize_col(&mut self, col: &crate::ColumnRef, value: rdbc::Value) -> anyhow::Result<()> {
        self.cols.push(rdbc::Arg {
            pos: rdbc::Placeholder::Name(col.name.to_owned()),
            value,
        });

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
