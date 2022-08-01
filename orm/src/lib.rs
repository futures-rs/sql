mod error;

pub use error::*;

mod ser;

mod der;

pub use der::*;
pub use ser::*;

pub use anyhow;

/// Execute statement with serede object
pub async fn execute<S>(stmt: &mut rdbc::Statement, value: &S) -> Result<rdbc::ExecuteResult>
where
    S: ?Sized + serde::Serialize,
{
    let args = {
        let mut serializer = OrmSerializer::default();

        value.serialize(&mut serializer)?;

        serializer.args
    };

    use serde::ser::Error;

    stmt.execute(args).await.map_err(Error::custom)
}

/// Query statement with serede object
pub async fn query<S>(stmt: &mut rdbc::Statement, value: &S) -> Result<rdbc::Rows>
where
    S: ?Sized + serde::Serialize,
{
    let args = {
        let mut serializer = OrmSerializer::default();

        value.serialize(&mut serializer)?;

        serializer.args
    };

    use serde::ser::Error;

    stmt.query(args).await.map_err(Error::custom)
}
