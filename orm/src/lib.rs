mod error;

pub use error::*;

mod ser;

mod der;

pub use der::*;
pub use ser::*;

pub use anyhow;

pub trait OrmStatment {
    fn execute<S>(&mut self, value: &S) -> rdbc::driver::Execute
    where
        S: ?Sized + serde::Serialize;

    fn query<S>(
        &mut self,
        value: &S,
    ) -> rdbc::WakableMapFuture<
        anyhow::Result<rdbc::Rows>,
        anyhow::Result<Box<dyn rdbc::driver::Rows>>,
    >
    where
        S: ?Sized + serde::Serialize;
}

impl OrmStatment for rdbc::Statement {
    /// Execute statement with serede object
    fn execute<S>(&mut self, value: &S) -> rdbc::driver::Execute
    where
        S: ?Sized + serde::Serialize,
    {
        let args = {
            let mut serializer = OrmSerializer::default();

            match value.serialize(&mut serializer) {
                Err(err) => {
                    let (fut, waker) = rdbc::driver::Execute::new();

                    waker.lock().unwrap().ready(Err(anyhow::Error::new(err)));

                    return fut;
                }
                _ => {}
            };

            serializer.args
        };

        self.execute(args)
    }

    /// Query statement with serede object
    fn query<S>(
        &mut self,
        value: &S,
    ) -> rdbc::WakableMapFuture<
        anyhow::Result<rdbc::Rows>,
        anyhow::Result<Box<dyn rdbc::driver::Rows>>,
    >
    where
        S: ?Sized + serde::Serialize,
    {
        let args = {
            let mut serializer = OrmSerializer::default();

            match value.serialize(&mut serializer) {
                Err(err) => {
                    let (fut, waker) = rdbc::WakableMapFuture::new();

                    waker.lock().unwrap().ready(Err(anyhow::Error::new(err)));

                    return fut;
                }
                _ => {}
            }

            serializer.args
        };

        self.query(args)
    }
}
