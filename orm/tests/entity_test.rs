use rdbc_attributes::*;
use rdbc_orm::{schema, Column, Deserialize, Serialize};

#[test]
fn test_table_declare() {
    #[allow(dead_code)]
    #[derive(Table)]
    #[table_name(color_table)]
    struct Color<Data> {
        #[col_primary]
        id: Column<u32>,

        #[col_unique(rgb_color)]
        #[col_name(rgb_color)]
        color: rdbc_orm::Column<Data>,

        #[col_unique(rgb_color)]
        gray: Column<u32>,

        #[col_index(color)]
        date: Column<u32>,
    }

    assert_eq!(Color::<i32>::column_color().col_name(), "rgb_color");

    assert_eq!(Color::<i32>::column_id().col_name(), "id");

    assert_eq!(Color::<u64>::table_name(), "color_table");

    assert_eq!(
        Color::<u64>::idx_col_primary().index_type,
        schema::IndexType::Primary
    );

    assert_eq!(
        Color::<u64>::idx_rgb_color().index_type,
        schema::IndexType::Unique
    );

    assert_eq!(Color::<u64>::idx_rgb_color().for_columns.len(), 2);
}
