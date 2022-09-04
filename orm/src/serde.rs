use std::sync::Arc;

use super::schema::*;

pub trait Entity {
    fn schema() -> &'static TableRef;
}

/// Normal column field trait
pub trait ColumnValue {
    fn rdbc_type() -> rdbc::ColumnType;
}

/// ORM object serialize context structure
pub trait Serializer {
    fn serialize_col(&mut self, col: &ColumnRef, value: rdbc::Value) -> anyhow::Result<()>;

    fn serialize_join_to<Join>(
        &mut self,
        col: &ColumnRef,
        join: Vec<Arc<Join>>,
    ) -> anyhow::Result<()>
    where
        Join: Entity + Serializable + 'static;
}

pub trait Deserializer {
    fn deserialize_col(&mut self, col: &ColumnRef) -> anyhow::Result<Option<rdbc::Value>>;

    fn deserialize_join_to<Join>(
        &mut self,
        col: &ColumnRef,
    ) -> anyhow::Result<Option<Vec<Arc<Join>>>>
    where
        Join: Entity + Deserializable + 'static;
}

pub trait Serializable {
    fn serialize<S>(&self, col: &ColumnRef, s: &mut S) -> anyhow::Result<()>
    where
        S: Serializer;
}

pub trait Deserializable: Sized {
    fn dserialize<D>(col: &ColumnRef, d: &mut D) -> anyhow::Result<Option<Self>>
    where
        D: Deserializer;
}

macro_rules! declare_col_int_type {
    ($t:ty) => {
        impl Serializable for $t {
            fn serialize<S>(&self, col: &ColumnRef, s: &mut S) -> anyhow::Result<()>
            where
                S: Serializer,
            {
                s.serialize_col(col, rdbc::Value::I64((*self) as i64))
            }
        }

        impl ColumnValue for $t {
            fn rdbc_type() -> rdbc::ColumnType {
                rdbc::ColumnType::I64
            }
        }

        impl Deserializable for $t {
            fn dserialize<D>(col: &ColumnRef, der: &mut D) -> anyhow::Result<Option<Self>>
            where
                D: Deserializer,
            {
                if let Some(rdbc::Value::I64(i)) = der.deserialize_col(col)? {
                    return Ok(Some(i as $t));
                }

                Ok(None)
            }
        }
    };
}

macro_rules! declare_col_int_types {
    ($($t:ty),*) => {
        $(declare_col_int_type!($t);)*
    };
}

declare_col_int_types!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! declare_col_float_type {
    ($t:ty) => {
        impl Serializable for $t {
            fn serialize<S>(&self, col: &ColumnRef, s: &mut S) -> anyhow::Result<()>
            where
                S: Serializer,
            {
                s.serialize_col(col, rdbc::Value::F64((*self) as f64))
            }
        }

        impl ColumnValue for $t {
            fn rdbc_type() -> rdbc::ColumnType {
                rdbc::ColumnType::F64
            }
        }

        impl Deserializable for $t {
            fn dserialize<D>(col: &ColumnRef, der: &mut D) -> anyhow::Result<Option<Self>>
            where
                D: Deserializer,
            {
                if let Some(rdbc::Value::F64(i)) = der.deserialize_col(col)? {
                    return Ok(Some(i as $t));
                }

                Ok(None)
            }
        }
    };
}

declare_col_float_type!(f32);
declare_col_float_type!(f64);

impl Serializable for String {
    fn serialize<S>(&self, col: &ColumnRef, s: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        s.serialize_col(col, rdbc::Value::String(self.clone()))
    }
}

impl ColumnValue for String {
    fn rdbc_type() -> rdbc::ColumnType {
        rdbc::ColumnType::String
    }
}

impl Deserializable for String {
    fn dserialize<D>(col: &ColumnRef, der: &mut D) -> anyhow::Result<Option<Self>>
    where
        D: Deserializer,
    {
        if let Some(rdbc::Value::String(i)) = der.deserialize_col(col)? {
            return Ok(Some(i));
        }

        Ok(None)
    }
}

impl Serializable for Vec<u8> {
    fn serialize<S>(&self, col: &ColumnRef, s: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        s.serialize_col(col, rdbc::Value::Bytes(self.clone()))
    }
}

impl ColumnValue for Vec<u8> {
    fn rdbc_type() -> rdbc::ColumnType {
        rdbc::ColumnType::Bytes
    }
}

impl Deserializable for Vec<u8> {
    fn dserialize<D>(col: &ColumnRef, der: &mut D) -> anyhow::Result<Option<Self>>
    where
        D: Deserializer,
    {
        if let Some(rdbc::Value::Bytes(i)) = der.deserialize_col(col)? {
            return Ok(Some(i));
        }

        Ok(None)
    }
}
