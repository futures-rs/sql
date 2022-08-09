/// Indexer declare object
pub struct IndexDef<'a> {
    pub name: &'a str, // index name, multicolumn indexer using the same name
    pub for_columns: &'a [&'a str], // indexer declare column name
    pub index_type: IndexType,
}

#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index,
    Unique,
    Primary,
    PrimaryAutoInc,
}
