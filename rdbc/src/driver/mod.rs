mod connection;
mod driver;
mod statement;
mod transaction;

pub use connection::*;
pub use driver::*;
pub use statement::*;
pub use transaction::*;

use anyhow::*;

/// Driver async tasks
pub enum Task {
    /// Prepare(id, query, waker)
    Prepare(
        String,
        String,
        futures_signal::Sender<Result<Box<dyn Statement>>>,
    ),
    /// (id, waker)
    Begin(String, futures_signal::Sender<Result<Box<dyn Transaction>>>),

    /// Open new connection (url, waker)
    Open(String, futures_signal::Sender<Result<Box<dyn Connection>>>),

    /// (stmt id, args, waker)
    Execute(
        String,
        Vec<Arg>,
        futures_signal::Sender<Result<ExecuteResult>>,
    ),

    /// (stmt id, args, waker)
    Query(
        String,
        Vec<Arg>,
        futures_signal::Sender<Result<Box<dyn Rows>>>,
    ),

    /// (resultset id, waker)
    Columns(String, futures_signal::Sender<Result<Vec<ColumnMetaData>>>),

    /// Iterate to next row (resultset id, waker)
    RowsNext(String, futures_signal::Sender<Result<bool>>),

    /// Current row get value by column index (resultset id, column index,column fetch type, waker)
    RowsGet(
        String,
        Placeholder,
        ColumnType,
        futures_signal::Sender<Result<Value>>,
    ),

    /// Transaction prepare (tx id, query, waker)
    TxPrepare(
        String,
        String,
        futures_signal::Sender<Result<Box<dyn Statement>>>,
    ),

    /// Commit tx (tx id,waker)
    Commit(String, futures_signal::Sender<Result<()>>),

    /// Rollback tx (tx id,waker)
    Rollback(String, futures_signal::Sender<Result<()>>),

    /// Close connection (connection id)
    CloseConnection(String),

    /// Close tx (tx id)
    CloseTx(String),

    /// Close stmt (stmt id)
    CloseStmt(String),

    /// Close resultset (resultset id)
    CloseRows(String),
}
