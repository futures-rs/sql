use crate::schema;

pub trait Serializer {
    fn serialize_col<V>(&mut self, idx: u32, value: &V) -> anyhow::Result<()>
    where
        V: Value;

    fn serialize_table<T: Table>(&mut self, idx: u32, t: &T) -> anyhow::Result<()>;
}

pub trait Deserializer {
    fn deserialize_col(&self);
}

pub trait Value {
    fn is_none() -> bool;
    fn to_rdbc_value(&self) -> rdbc::Value;

    fn from_rdbc_value(&mut self, value: rdbc::Value) -> anyhow::Result<()>;
}

pub trait Table {
    fn schema() -> &'static schema::TableRef;

    fn is_none() -> bool;

    fn serialize<Ser>(&self, s: Ser) -> anyhow::Result<()>
    where
        Ser: Serializer;

    fn deserialize<Der>(&mut self, der: Der) -> anyhow::Result<()>
    where
        Der: Deserializer;
}
