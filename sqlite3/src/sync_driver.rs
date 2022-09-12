use super::sqlite3_rs;
use rdbc::driver;

pub struct SyncDriver {}

impl driver::Driver for SyncDriver {
    fn open(&mut self, name: &str) -> driver::Connector {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(sqlite3_rs::Connection::open(name).map(|c| SyncConnection { inner: c }.into()));

        fut
    }
}

struct SyncConnection {
    inner: sqlite3_rs::Connection,
}

unsafe impl Send for SyncConnection {}

impl Into<Box<dyn driver::Connection>> for SyncConnection {
    fn into(self) -> Box<dyn driver::Connection> {
        Box::new(self)
    }
}

impl driver::Connection for SyncConnection {
    fn begin(&mut self) -> driver::Begin {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(
            self.inner
                .begin()
                .map(|tx| SyncTransaction { inner: tx }.into()),
        );

        fut
    }

    fn id(&self) -> &str {
        &self.inner.id
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(
            self.inner
                .prepare(query)
                .map(|stmt| SyncStatement { inner: stmt }.into()),
        );

        fut
    }
}

struct SyncTransaction {
    inner: sqlite3_rs::Transaction,
}

impl Into<Box<dyn driver::Transaction>> for SyncTransaction {
    fn into(self) -> Box<dyn driver::Transaction> {
        Box::new(self)
    }
}

impl driver::Transaction for SyncTransaction {
    fn commit(&mut self) -> driver::Commit {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.commit());

        fut
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(
            self.inner
                .prepare(query)
                .map(|stmt| SyncStatement { inner: stmt }.into()),
        );

        fut
    }

    fn rollback(&mut self) -> driver::Rollback {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.rollback());
        fut
    }
}

struct SyncStatement {
    inner: sqlite3_rs::Statement,
}

impl Into<Box<dyn driver::Statement>> for SyncStatement {
    fn into(self) -> Box<dyn driver::Statement> {
        Box::new(self)
    }
}

unsafe impl Send for SyncStatement {}

impl driver::Statement for SyncStatement {
    fn execute(&mut self, args: Vec<rdbc::Arg>) -> driver::Execute {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.execute(args));

        fut
    }

    fn num_input(&self) -> Option<u32> {
        self.inner.num_input()
    }

    fn query(&mut self, args: Vec<rdbc::Arg>) -> driver::Query {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(
            self.inner
                .query(args)
                .map(|rows| SyncRows { inner: rows }.into()),
        );

        fut
    }
}

struct SyncRows {
    inner: sqlite3_rs::Rows,
}

impl Into<Box<dyn driver::Rows>> for SyncRows {
    fn into(self) -> Box<dyn driver::Rows> {
        Box::new(self)
    }
}

unsafe impl Send for SyncRows {}

impl driver::Rows for SyncRows {
    fn colunms(&mut self) -> driver::Columns {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.colunms().map(|c| c.clone()));

        fut
    }

    fn get(
        &mut self,
        index: driver::Placeholder,
        column_type: driver::ColumnType,
    ) -> driver::RowsGet {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.get(index, column_type));

        fut
    }

    fn next(&mut self) -> driver::RowsNext {
        let (fut, waker) = rdbc::futures_signal::cond();

        waker.ready(self.inner.next());

        fut
    }
}
