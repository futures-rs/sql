use std::marker::PhantomData;

use crate::{Entity, Serializer};

use anyhow::Result;

pub struct InsertSerializer<E> {
    _marker: PhantomData<E>,
}

impl<E> InsertSerializer<E> {
    /// Create new insert serializer
    pub fn new() -> Result<Self> {
        Ok(InsertSerializer {
            _marker: Default::default(),
        })
    }
}

impl<E> Serializer for InsertSerializer<E>
where
    E: Entity,
{
    fn serialize_col(&mut self, col: &crate::ColumnRef, value: rdbc::Value) -> anyhow::Result<()> {
        Ok(())
    }

    fn serialize_join_to<Join>(
        &mut self,
        col: &crate::ColumnRef,
        join: Vec<std::sync::Arc<Join>>,
    ) -> anyhow::Result<()>
    where
        Join: Entity + crate::Serializable,
    {
        Ok(())
    }
}
