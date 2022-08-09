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

#[derive(Debug, Default)]
pub struct Column<Data> {
    _data: Option<Data>,
    _value: Option<rdbc::Value>,
    _column: Option<schema::ColumnDef<Data>>,
}

impl<Data> Serialize for Column<Data>
where
    Data: ColumnValue<ColumnType = Data> + Default,
{
    fn orm_seralize<S>(&mut self, _ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        self._data.cast_to_rdbc_value()?;

        Ok(())
    }
}

impl<Data> Deserialize for Column<Data>
where
    Data: ColumnValue<ColumnType = Data> + Default,
{
    fn orm_deseralize<D>(&mut self, _der: &mut D) -> anyhow::Result<()>
    where
        D: Deserializer,
    {
        Ok(())
    }
}
