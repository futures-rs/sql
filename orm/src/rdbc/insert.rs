use std::{marker::PhantomData, sync::Arc};

use crate::{Entity, Serializer};

use anyhow::Result;
use rdbc::DataSource;

trait InsertJoinSerializer {}

pub struct InsertSerializer<E> {
    _marker: PhantomData<E>,
    _values: Vec<rdbc::Arg>,
    _joins: Vec<Box<dyn InsertJoinSerializer>>,
}

impl<E> InsertSerializer<E>
where
    E: Entity,
{
    /// Create new insert serializer
    pub fn new() -> Result<Self> {
        Ok(InsertSerializer {
            _marker: Default::default(),
            _values: Default::default(),
            _joins: Default::default(),
        })
    }

    pub fn save(_database: &mut DataSource) -> Result<()> {
        Ok(())
    }

    /// check if target col exists
    #[inline]
    fn col_exists(&self, col: &crate::ColumnRef) -> bool {
        for target in &E::schema().columns {
            if target.name == col.name {
                return true;
            }
        }

        false
    }
}

impl<E> Serializer for InsertSerializer<E>
where
    E: Entity,
{
    fn serialize_col(&mut self, col: &crate::ColumnRef, value: rdbc::Value) -> anyhow::Result<()> {
        if cfg!(runtime_check) {
            if !self.col_exists(col) {
                return Err(anyhow::format_err!("col not exists : {}", col));
            }
        }

        self._values.push(rdbc::Arg {
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
        Join: Entity + crate::Serializable + 'static,
    {
        if cfg!(runtime_check) {
            if !self.col_exists(col) {
                return Err(anyhow::format_err!("col not exists : {}", col));
            }
        }

        self._joins.push(InsertJoinSerializerImpl::new(join));

        Ok(())
    }
}

#[allow(dead_code)]
struct InsertJoinSerializerImpl<Join> {
    join: Vec<Arc<Join>>,
}

impl<Join> InsertJoinSerializerImpl<Join>
where
    Join: Entity + crate::Serializable + 'static,
{
    fn new(join: Vec<std::sync::Arc<Join>>) -> Box<dyn InsertJoinSerializer> {
        Box::new(InsertJoinSerializerImpl { join })
    }
}

impl<Join> InsertJoinSerializer for InsertJoinSerializerImpl<Join> {}
