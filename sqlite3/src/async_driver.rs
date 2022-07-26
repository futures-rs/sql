use super::sqlite3_rs;
use rdbc::driver;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

#[allow(dead_code)]
pub struct AsyncDriver {
    sender: Sender<driver::Task>,
}

fn fetch_object<'a, Obj, Output>(
    waker: &rdbc::SharedWaker<anyhow::Result<Output>>,
    map: &'a mut HashMap<String, Obj>,
    id: &str,
) -> Option<&'a mut Obj> {
    let obj = map.get_mut(id);

    if obj.is_none() {
        waker
            .lock()
            .unwrap()
            .ready(Err(anyhow::anyhow!("sqlite3 resource not found {}", id)));
    }

    obj
}

impl AsyncDriver {
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        let execute_loop_sender = sender.clone();

        std::thread::spawn(move || Self::execute_loop(execute_loop_sender, receiver));

        Self { sender }
    }

    fn execute_loop(
        sender: Sender<driver::Task>,
        receiver: Receiver<driver::Task>,
    ) -> anyhow::Result<()> {
        let mut cnns = HashMap::<String, sqlite3_rs::Connection>::new();
        let mut stmts = HashMap::<String, sqlite3_rs::Statement>::new();
        let mut txs = HashMap::<String, sqlite3_rs::Transaction>::new();
        let mut results = HashMap::<String, sqlite3_rs::Rows>::new();

        loop {
            match receiver.recv()? {
                driver::Task::Begin(id, waker) => {
                    if let Some(conn) = fetch_object(&waker, &mut cnns, &id) {
                        waker.lock().unwrap().ready(conn.begin().map(|tx| {
                            {
                                let id = tx.id.clone();

                                txs.insert(tx.id.clone(), tx);

                                AsyncTransaction {
                                    sender: sender.clone(),
                                    id,
                                }
                            }
                            .into()
                        }));
                    }
                }

                driver::Task::CloseConnection(id) => {
                    cnns.remove(&id);
                }

                driver::Task::Prepare(id, query, waker) => {
                    if let Some(conn) = fetch_object(&waker, &mut cnns, &id) {
                        waker.lock().unwrap().ready(conn.prepare(&query).map(|obj| {
                            {
                                let id = obj.id.clone();

                                let inputs = obj.num_input();

                                stmts.insert(obj.id.clone(), obj);

                                AsyncStatement {
                                    sender: sender.clone(),
                                    id,
                                    inputs,
                                }
                            }
                            .into()
                        }));
                    }
                }

                driver::Task::Open(url, waker) => match sqlite3_rs::Connection::open(&url) {
                    Ok(conn) => {
                        let id = conn.id.clone();

                        cnns.insert(id.clone(), conn);

                        waker.lock().unwrap().ready(Ok(AsyncConnection {
                            id,
                            sender: sender.clone(),
                        }
                        .into()));
                    }
                    Err(err) => {
                        waker.lock().unwrap().ready(Err(err));
                    }
                },

                driver::Task::Execute(id, args, waker) => {
                    if let Some(stmt) = fetch_object(&waker, &mut stmts, &id) {
                        waker.lock().unwrap().ready(stmt.execute(args));
                    }
                }

                driver::Task::Query(id, args, waker) => {
                    if let Some(stmt) = fetch_object(&waker, &mut stmts, &id) {
                        waker.lock().unwrap().ready(stmt.query(args).map(|obj| {
                            {
                                let id = obj.id.clone();

                                results.insert(obj.id.clone(), obj);

                                AsyncRows {
                                    sender: sender.clone(),
                                    id,
                                }
                            }
                            .into()
                        }));
                    }
                }

                driver::Task::Columns(id, waker) => {
                    if let Some(rows) = fetch_object(&waker, &mut results, &id) {
                        waker.lock().unwrap().ready(rows.colunms());
                    }
                }

                driver::Task::RowsNext(id, waker) => {
                    if let Some(rows) = fetch_object(&waker, &mut results, &id) {
                        waker.lock().unwrap().ready(rows.next());
                    }
                }

                driver::Task::RowsGet(id, index, col_type, waker) => {
                    if let Some(rows) = fetch_object(&waker, &mut results, &id) {
                        waker.lock().unwrap().ready(rows.get(index, col_type));
                    }
                }

                driver::Task::TxPrepare(id, query, waker) => {
                    if let Some(tx) = fetch_object(&waker, &mut txs, &id) {
                        waker.lock().unwrap().ready(tx.prepare(&query).map(|obj| {
                            {
                                let id = obj.id.clone();

                                let inputs = obj.num_input();

                                stmts.insert(obj.id.clone(), obj);

                                AsyncStatement {
                                    sender: sender.clone(),
                                    id,
                                    inputs,
                                }
                            }
                            .into()
                        }));
                    }
                }

                driver::Task::Commit(id, waker) => {
                    if let Some(tx) = fetch_object(&waker, &mut txs, &id) {
                        waker.lock().unwrap().ready(tx.commit());
                    }
                }

                driver::Task::Rollback(id, waker) => {
                    if let Some(tx) = fetch_object(&waker, &mut txs, &id) {
                        waker.lock().unwrap().ready(tx.rollback());
                    }
                }

                driver::Task::CloseTx(id) => {
                    txs.remove(&id);
                }

                driver::Task::CloseStmt(id) => {
                    stmts.remove(&id);
                }

                driver::Task::CloseRows(id) => {
                    results.remove(&id);
                }
            }
        }
    }
}

fn send_task<Output>(
    sender: &mut Sender<driver::Task>,
    waker: rdbc::SharedWaker<anyhow::Result<Output>>,
    task: driver::Task,
) {
    if let Err(err) = sender.send(task) {
        waker.lock().unwrap().ready(Err(anyhow::Error::new(err)));
    }
}

impl driver::Driver for AsyncDriver {
    fn open(&mut self, name: &str) -> driver::Connector {
        let (fut, waker) = driver::Connector::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Open(name.to_owned(), waker),
        );

        fut
    }
}

struct AsyncConnection {
    sender: Sender<driver::Task>,
    id: String,
}

impl Into<Box<dyn driver::Connection>> for AsyncConnection {
    fn into(self) -> Box<dyn driver::Connection> {
        Box::new(self)
    }
}

impl Drop for AsyncConnection {
    fn drop(&mut self) {
        _ = self
            .sender
            .send(driver::Task::CloseConnection(self.id.clone()));
    }
}

impl driver::Connection for AsyncConnection {
    fn begin(&mut self) -> driver::Begin {
        let (fut, waker) = driver::Begin::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Begin(self.id.clone(), waker),
        );

        fut
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = driver::Prepare::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Prepare(self.id.clone(), query.to_owned(), waker),
        );

        fut
    }
}

struct AsyncTransaction {
    sender: Sender<driver::Task>,
    id: String,
}

impl Drop for AsyncTransaction {
    fn drop(&mut self) {
        _ = self.sender.send(driver::Task::CloseTx(self.id.clone()));
    }
}

impl Into<Box<dyn driver::Transaction>> for AsyncTransaction {
    fn into(self) -> Box<dyn driver::Transaction> {
        Box::new(self)
    }
}

impl driver::Transaction for AsyncTransaction {
    fn commit(&mut self) -> driver::Commit {
        let (fut, waker) = driver::Commit::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Commit(self.id.clone(), waker),
        );

        fut
    }

    fn prepare(&mut self, query: &str) -> driver::Prepare {
        let (fut, waker) = driver::Prepare::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::TxPrepare(self.id.clone(), query.to_owned(), waker),
        );

        fut
    }

    fn rollback(&mut self) -> driver::Rollback {
        let (fut, waker) = driver::Rollback::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Rollback(self.id.clone(), waker),
        );

        fut
    }
}

struct AsyncStatement {
    sender: Sender<driver::Task>,
    id: String,
    inputs: Option<u32>,
}

impl Drop for AsyncStatement {
    fn drop(&mut self) {
        _ = self.sender.send(driver::Task::CloseStmt(self.id.clone()));
    }
}

unsafe impl Send for AsyncStatement {}

impl Into<Box<dyn driver::Statement>> for AsyncStatement {
    fn into(self) -> Box<dyn driver::Statement> {
        Box::new(self)
    }
}

impl driver::Statement for AsyncStatement {
    fn execute(&mut self, args: Vec<rdbc::NamedValue>) -> driver::Execute {
        let (fut, waker) = driver::Execute::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Execute(self.id.clone(), args, waker),
        );

        fut
    }

    fn num_input(&self) -> Option<u32> {
        self.inputs
    }

    fn query(&mut self, args: Vec<rdbc::NamedValue>) -> driver::Query {
        let (fut, waker) = driver::Query::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Query(self.id.clone(), args, waker),
        );

        fut
    }
}

struct AsyncRows {
    sender: Sender<driver::Task>,
    id: String,
}

impl Drop for AsyncRows {
    fn drop(&mut self) {
        _ = self.sender.send(driver::Task::CloseRows(self.id.clone()));
    }
}

impl Into<Box<dyn driver::Rows>> for AsyncRows {
    fn into(self) -> Box<dyn driver::Rows> {
        Box::new(self)
    }
}

unsafe impl Send for AsyncRows {}

impl driver::Rows for AsyncRows {
    fn colunms(&mut self) -> driver::Columns {
        let (fut, waker) = driver::Columns::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::Columns(self.id.clone(), waker),
        );

        fut
    }

    fn get(&mut self, index: u64, column_type: driver::ColumnType) -> driver::RowsGet {
        let (fut, waker) = driver::RowsGet::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::RowsGet(self.id.clone(), index, column_type, waker),
        );

        fut
    }

    fn next(&mut self) -> driver::RowsNext {
        let (fut, waker) = driver::RowsNext::new();

        send_task(
            &mut self.sender,
            waker.clone(),
            driver::Task::RowsNext(self.id.clone(), waker),
        );

        fut
    }
}
