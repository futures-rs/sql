use rdbc_attributes::*;
use rdbc_orm::Column;

#[test]
fn test_table_declare() {
    #[derive(Debug, Table)]
    struct Palette {
        #[col_primary]
        id: Column<i32>,
    }
}
