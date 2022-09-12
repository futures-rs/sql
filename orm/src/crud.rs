use rdbc::Preparable;

use crate::Table;

#[allow(dead_code)]
/// Data insert builder
pub struct CRUD<P> {
    prepare: P, // database for insert
}

impl<P> CRUD<P> {
    pub fn new(prepare: P) -> Self {
        Self { prepare }
    }
}

impl<P> CRUD<P>
where
    P: Preparable,
{
    pub fn insert<T>(t: &T) -> anyhow::Result<()>
    where
        T: Table,
    {
        let table_ref = T::schema();

        Ok(())
    }
}
