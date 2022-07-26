use super::sqlite3_rs;
use rdbc::driver;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct AsyncDriver {
    receiver: Receiver<driver::Task>,
    sender: Sender<driver::Task>,
}

impl AsyncDriver {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl driver::Driver for AsyncDriver {
    fn open(&mut self, name: &str) -> driver::Connector {
        let (fut, waker) = driver::Connector::new();

        waker
            .lock()
            .unwrap()
            .ready(sqlite3_rs::Connection::open(name).map(|c| AsyncConnection { inner: c }.into()));

        fut
    }
}

struct AsyncConnection {
    inner: sqlite3_rs::Connection,
}

impl Into<Box<dyn driver::Connection>> for AsyncConnection {
    fn into(self) -> Box<dyn driver::Connection> {
        Box::new(self)
    }
}

impl driver::Connection for AsyncConnection {
    fn begin(&mut self) -> driver::Begin {
        let (fut, waker) = driver::Begin::new();

        waker.lock().unwrap().ready(
            self.inner
                .begin()
                .map(|tx| AsyncTransaction { inner: tx }.into()),
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
        let (fut, waker) = driver::Prepare::new();

        waker.lock().unwrap().ready(
            self.inner
                .prepare(query)
                .map(|stmt| AsyncStatement { inner: stmt }.into()),
        );

        fut
    }
}

struct AsyncTransaction {
    inner: sqlite3_rs::Transaction,
}

impl Into<Box<dyn driver::Transaction>> for AsyncTransaction {
    fn into(self) -> Box<dyn driver::Transaction> {
        Box::new(self)
    }
}

impl driver::Transaction for AsyncTransaction {
    fn commit(&mut self) -> driver::Commit {
        let (fut, waker) = driver::Commit::new();

        waker.lock().unwrap().ready(self.inner.commit());

        fut
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = driver::Prepare::new();

        waker.lock().unwrap().ready(
            self.inner
                .prepare(query)
                .map(|stmt| AsyncStatement { inner: stmt }.into()),
        );

        fut
    }

    fn rollback(&mut self) -> driver::Rollback {
        let (fut, waker) = driver::Rollback::new();

        waker.lock().unwrap().ready(self.inner.rollback());
        fut
    }
}

struct AsyncStatement {
    inner: sqlite3_rs::Statement,
}

impl Into<Box<dyn driver::Statement>> for AsyncStatement {
    fn into(self) -> Box<dyn driver::Statement> {
        Box::new(self)
    }
}

impl driver::Statement for AsyncStatement {
    fn execute(&mut self, args: Vec<rdbc::NamedValue>) -> driver::Execute {
        let (fut, waker) = driver::Execute::new();

        waker.lock().unwrap().ready(self.inner.execute(args));

        fut
    }

    fn num_input(&self) -> Option<u32> {
        self.inner.num_input()
    }

    fn query(&mut self, args: Vec<rdbc::NamedValue>) -> driver::Query {
        let (fut, waker) = driver::Query::new();

        waker.lock().unwrap().ready(
            self.inner
                .query(args)
                .map(|rows| AsyncRows { inner: rows }.into()),
        );

        fut
    }
}

struct AsyncRows {
    inner: sqlite3_rs::Rows,
}

impl Into<Box<dyn driver::Rows>> for AsyncRows {
    fn into(self) -> Box<dyn driver::Rows> {
        Box::new(self)
    }
}

impl driver::Rows for AsyncRows {
    fn colunms(&mut self) -> driver::Columns {
        let (fut, waker) = driver::Columns::new();

        waker.lock().unwrap().ready(self.inner.colunms());

        fut
    }

    fn get(&mut self, index: u64, column_type: driver::ColumnType) -> driver::RowsGet {
        let (fut, waker) = driver::RowsGet::new();

        waker
            .lock()
            .unwrap()
            .ready(self.inner.get(index, column_type));

        fut
    }

    fn next(&mut self) -> driver::RowsNext {
        let (fut, waker) = driver::RowsNext::new();

        waker.lock().unwrap().ready(self.inner.next());

        fut
    }
}
