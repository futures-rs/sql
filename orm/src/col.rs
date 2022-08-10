use super::ser::Serializer;

use super::der::Deserializer;

use super::schema;

mod value;

pub use value::*;

pub trait Serialize {
    fn orm_seralize<S>(&mut self, ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer;
}

pub trait Deserialize {
    fn orm_deseralize<D>(&mut self, der: &mut D) -> anyhow::Result<()>
    where
        D: Deserializer;
}

impl<T> Serialize for Option<T>
where
    T: Serialize + Default,
{
    fn orm_seralize<S>(&mut self, ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        if let Some(t) = self.as_mut() {
            return t.orm_seralize(ser);
        }

        Ok(())
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize + Default,
{
    fn orm_seralize<S>(&mut self, ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
    }
}

#[derive(Debug, Default)]
pub struct Column<Data> {
    _data: Option<Data>,
    _value: Option<rdbc::Value>,
    _column: Option<schema::ColumnDef<Data>>,
}

impl<Data> Serialize for Column<Data>
where
    Data: Serialize + Deserialize + Default,
{
    fn orm_seralize<S>(&mut self, ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        self._data.orm_seralize(ser)?;

        Ok(())
    }
}

impl<Data> Deserialize for Column<Data>
where
    Data: ColumnValue<ColumnType = Data> + Default,
{
    fn orm_deseralize<D>(&mut self, der: &mut D) -> anyhow::Result<()>
    where
        D: Deserializer,
    {
        self._data.orm_deseralize(der)?;

        Ok(())
    }
}
