use std::marker::PhantomData;

use super::ser::Serializer;

use super::der::Deserializer;

use super::schema;

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
    _marker: PhantomData<Data>,
}

impl<Data> Serialize for Column<Data> {
    fn orm_seralize<S>(&mut self, _ser: &mut S) -> anyhow::Result<()>
    where
        S: Serializer,
    {
        Ok(())
    }
}

impl<Data> Deserialize for Column<Data> {
    fn orm_deseralize<D>(&mut self, _der: &mut D) -> anyhow::Result<()>
    where
        D: Deserializer,
    {
        Ok(())
    }
}

impl<Data> Column<Data> {
    pub fn column_def_static(name: &str) -> schema::ColumnDef<Data> {
        schema::ColumnDef::<Data>::new(name)
    }

    pub fn column_def(&self, name: &str) -> schema::ColumnDef<Data> {
        Self::column_def_static(name)
    }
}
