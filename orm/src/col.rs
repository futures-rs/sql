use anyhow::{Ok, Result};

/// ORM data format can serialize any data structure supported by rdbc-orm
pub trait Serializer {
    fn write_rdbc_value(&mut self, ph: rdbc::Placeholder, value: rdbc::Value)
        -> anyhow::Result<()>;
}

/// ORM data format can deserialize any data structure supported by rdbc-orm
pub trait Deserializer {
    fn read_rdbc_value(&mut self, ph: rdbc::Placeholder) -> anyhow::Result<Option<rdbc::Value>>;
}

/// ORM column value trait
pub trait ColumnValue {
    fn rdbc_type() -> rdbc::ColumnType;
}

/// Indicate target object can be serializing by orm
pub trait Serializable {
    fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
    where
        S: Serializer;
}

/// Indicate target object can be deserializing by orm
pub trait Deserializable: Sized {
    fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
    where
        D: Deserializer;
}

#[derive(Debug)]
/// ORM table declare structure
pub struct Column<T> {
    _data: Option<T>,
}

impl<T> Column<T>
where
    T: ColumnValue,
{
    /// Get column rdbc type
    pub fn rdbc_type() -> rdbc::ColumnType {
        T::rdbc_type()
    }
}

/// Impl [`Serializable`] for [`Column<T>`]
impl<T> Serializable for Column<T>
where
    T: Serializable,
{
    fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
    where
        S: Serializer,
    {
        if let Some(t) = self._data.as_ref() {
            t.serialize(ph, s)
        } else {
            Ok(())
        }
    }
}

/// Impl [`Deserializable`] for [`Column<T>`]
impl<T> Deserializable for Column<T>
where
    T: Deserializable + Default,
{
    fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
    where
        D: Deserializer,
    {
        if let Some(t) = T::dserialize(ph, der)? {
            return Ok(Some(Column { _data: Some(t) }));
        }

        return Ok(None);
    }
}

macro_rules! declare_col_int_type {
    ($t:ty) => {
        impl Serializable for $t {
            fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
            where
                S: Serializer,
            {
                s.write_rdbc_value(ph, rdbc::Value::I64((*self) as i64))
            }
        }

        impl Deserializable for $t {
            fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
            where
                D: Deserializer,
            {
                if let Some(rdbc::Value::I64(i)) = der.read_rdbc_value(ph)? {
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
            fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
            where
                S: Serializer,
            {
                s.write_rdbc_value(ph, rdbc::Value::F64((*self) as f64))
            }
        }

        impl Deserializable for $t {
            fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
            where
                D: Deserializer,
            {
                if let Some(rdbc::Value::F64(i)) = der.read_rdbc_value(ph)? {
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
    fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
    where
        S: Serializer,
    {
        s.write_rdbc_value(ph, rdbc::Value::String(self.clone()))
    }
}

impl Deserializable for String {
    fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
    where
        D: Deserializer,
    {
        if let Some(rdbc::Value::String(i)) = der.read_rdbc_value(ph)? {
            return Ok(Some(i));
        }

        Ok(None)
    }
}

impl Serializable for Vec<u8> {
    fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
    where
        S: Serializer,
    {
        s.write_rdbc_value(ph, rdbc::Value::Bytes(self.clone()))
    }
}

impl Deserializable for Vec<u8> {
    fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
    where
        D: Deserializer,
    {
        if let Some(rdbc::Value::Bytes(i)) = der.read_rdbc_value(ph)? {
            return Ok(Some(i));
        }

        Ok(None)
    }
}
