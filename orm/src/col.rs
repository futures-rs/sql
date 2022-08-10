use anyhow::Result;

/// ORM data format can serialize any data structure supported by rdbc-orm
pub trait Serializer {}

/// ORM data format can deserialize any data structure supported by rdbc-orm
pub trait Deserializer {}

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

/// Impl Serializable for Option
impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn serialize<S>(&self, ph: rdbc::Placeholder, s: &mut S) -> Result<()>
    where
        S: Serializer,
    {
        if let Some(t) = self.as_ref() {
            t.serialize(ph, s)
        } else {
            Ok(())
        }
    }
}

/// Impl Deserializable for Option
// impl<T> Deserializable for Option<T>
// where
//     T: Deserializable + Default,
// {
//     fn dserialize<D>(ph: rdbc::Placeholder, der: &mut D) -> Result<Option<Self>>
//     where
//         D: Deserializer,
//     {
//         T::dserialize(ph, der).map(|r| r.map(|v| Some(v)))
//     }
// }

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
        self._data.serialize(ph, s)
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
