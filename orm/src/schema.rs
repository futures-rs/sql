pub trait TableRef {
    fn name() -> &'static str;
    fn columns() -> Vec<&'static ColumnRef>;
    fn indexes() -> Vec<&'static IndexRef>;
}

/// ORM rdbc table column define structure
pub struct ColumnRef {
    pub name: &'static str,                 // col name
    pub col_type: ColumnType,               // col type enum
    pub col_decltype: Option<&'static str>, // col sql declare type string
}

/// ORM column types
pub enum ColumnType {
    RDBC(rdbc::ColumnType), // column basic type
    OneToOne(JoinRef),      // one to one column orm type
    OneToMany(JoinRef),     // one to many column orm type
    ManyToMany(JoinRef),    // many to many column orm type
    ManyToOne(JoinRef),     // to help generate optimized sql
}

/// ORM join reference define structure
pub struct JoinRef {
    pub name: &'static str, // join statment unique name, to help generate join on multiple columns sql .
    pub rdbc_type: rdbc::ColumnType, // rdbc column type .
    pub join: &'static str, // join table name .
    pub to: &'static str,   // join table column name .
}

/// ORM table index define structure
pub struct IndexRef {
    pub name: &'static str, // table scope unique name, multiple columns index use this value to group
    pub idx_type: IndexType, // index type
    pub col_group: &'static [&'static str], // index includes columns
}

/// ORM table index type
pub enum IndexType {
    Index,          // normal index
    Primary,        // table primary key
    PrimaryAutoInc, // table auto increament primary key
    Unique,         // unique index
    Foreign,        // foreign key
}
