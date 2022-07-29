mod connection;
mod driver;
mod statement;
mod transaction;

pub use connection::*;
pub use driver::*;
pub use statement::*;
pub use transaction::*;

use crate::waker;

use anyhow::*;

/// Driver async tasks
pub enum Task {
    /// Prepare(id, query, waker)
    Prepare(
        String,
        String,
        waker::SharedWaker<Result<Box<dyn Statement>>>,
    ),
    /// (id, waker)
    Begin(String, waker::SharedWaker<Result<Box<dyn Transaction>>>),

    /// Open new connection (url, waker)
    Open(String, waker::SharedWaker<Result<Box<dyn Connection>>>),

    /// (stmt id, args, waker)
    Execute(String, Vec<Arg>, waker::SharedWaker<Result<ExecuteResult>>),

    /// (stmt id, args, waker)
    Query(String, Vec<Arg>, waker::SharedWaker<Result<Box<dyn Rows>>>),

    /// (resultset id, waker)
    Columns(String, waker::SharedWaker<Result<Vec<ColumnMetaData>>>),

    /// Iterate to next row (resultset id, waker)
    RowsNext(String, waker::SharedWaker<Result<bool>>),

    /// Current row get value by column index (resultset id, column index,column fetch type, waker)
    RowsGet(
        String,
        Placeholder,
        ColumnType,
        waker::SharedWaker<Result<Value>>,
    ),

    /// Transaction prepare (tx id, query, waker)
    TxPrepare(
        String,
        String,
        waker::SharedWaker<Result<Box<dyn Statement>>>,
    ),

    /// Commit tx (tx id,waker)
    Commit(String, waker::SharedWaker<Result<()>>),

    /// Rollback tx (tx id,waker)
    Rollback(String, waker::SharedWaker<Result<()>>),

    /// Close connection (connection id)
    CloseConnection(String),

    /// Close tx (tx id)
    CloseTx(String),

    /// Close stmt (stmt id)
    CloseStmt(String),

    /// Close resultset (resultset id)
    CloseRows(String),
}
