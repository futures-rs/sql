use anyhow::Result;

pub trait Serializer {
    fn next(&mut self, placehodler: rdbc::Placeholder) -> Result<()>;
}

pub trait Serialize {
    fn orm_seralize<S>(&self, ser: &mut S) -> Result<()>
    where
        S: Serializer;
}

impl Serialize for i32 {
    fn orm_seralize<S>(&self, _ser: &mut S) -> Result<()>
    where
        S: Serializer,
    {
        Ok(())
    }
}
