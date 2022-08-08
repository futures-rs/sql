/// Indexer declare object
pub struct IndexDef {
    pub name: String,             // index name, multicolumn indexer using the same name
    pub for_columns: Vec<String>, // indexer declare column name
    pub index_type: IndexType,
}

#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index,
    Unique,
    Primary,
    PrimaryAutoInc,
}
