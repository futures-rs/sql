pub struct ColumnDef {
    pub ph: rdbc::Placeholder,
    pub col_type: ColumnType,
    pub description: Option<&'static str>,
}

impl ColumnDef {
    pub const fn new(
        ph: rdbc::Placeholder,
        col_type: ColumnType,
        description: Option<&'static str>,
    ) -> Self {
        Self {
            ph,
            description,
            col_type,
        }
    }
}

pub enum ColumnType {
    RDBC(rdbc::ColumnType),
    OneToOne(OneToOne),
    OneToMany(OneToMany),
    ManyToMany(ManyToMany),
}

pub struct OneToOne {
    pub col_type: rdbc::ColumnType,
    pub to: &'static str,        // reference table name
    pub to_column: &'static str, // foreign key
}

pub struct OneToMany {}

pub struct ManyToMany {}

pub struct IndexDef {
    pub name: &'static str,
    pub idx_type: IndexType,
    pub columns: &'static [&'static str],
}

#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index,
    Unique,
    Primary,
    PrimaryAutoInc,
}
