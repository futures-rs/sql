use rdbc::Preparable;

use crate::Table;

use super::CRUD;

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

trait Test {
    type Bar<'a>;
}
