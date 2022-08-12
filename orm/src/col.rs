use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{ColumnValue, Deserializable, Serializable, TableRef};

/// ORM table normal field
pub struct Column<T>
where
    T: ColumnValue,
{
    pub data: Option<T>,
}

impl<T> Deref for Column<T>
where
    T: ColumnValue,
{
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Column<T>
where
    T: ColumnValue,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Serializable for Column<T>
where
    T: ColumnValue + Serializable,
{
    fn serialize<S>(&self, col: &crate::ColumnRef, s: &mut S) -> anyhow::Result<()>
    where
        S: crate::Serializer,
    {
        if self.is_some() {
            return self.as_ref().unwrap().serialize(col, s);
        } else {
            return Ok(());
        }
    }
}

impl<T> Deserializable for Column<T>
where
    T: ColumnValue + Deserializable,
{
    fn dserialize<D>(col: &crate::ColumnRef, d: &mut D) -> anyhow::Result<Option<Self>>
    where
        D: crate::Deserializer,
    {
        if let Some(t) = T::dserialize(col, d)? {
            return Ok(Some(Column { data: Some(t) }));
        } else {
            return Ok(None);
        }
    }
}

/// ORM table join to field define, specially for one to one relationship
pub struct OneToOne<T>
where
    T: TableRef,
{
    pub data: Option<Arc<T>>,
}

impl<T> Deref for OneToOne<T>
where
    T: TableRef,
{
    type Target = Option<Arc<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for OneToOne<T>
where
    T: TableRef,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// ORM table join to field define, specially for one to many relationship
pub struct OneToMany<T>
where
    T: TableRef,
{
    pub data: Option<Vec<Arc<T>>>,
}

impl<T> Deref for OneToMany<T>
where
    T: TableRef,
{
    type Target = Option<Vec<Arc<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for OneToMany<T>
where
    T: TableRef,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// ORM table join to field define, specially for many to many relationship
pub struct ManyToMany<T>
where
    T: TableRef,
{
    pub data: Option<Vec<Arc<T>>>,
}

impl<T> Deref for ManyToMany<T>
where
    T: TableRef,
{
    type Target = Option<Vec<Arc<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for ManyToMany<T>
where
    T: TableRef,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// ORM table join to field define, specially for many to one relationship
pub struct ManyToOne<T>
where
    T: TableRef,
{
    pub data: Option<Arc<T>>,
}

impl<T> Deref for ManyToOne<T>
where
    T: TableRef,
{
    type Target = Option<Arc<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for ManyToOne<T>
where
    T: TableRef,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
