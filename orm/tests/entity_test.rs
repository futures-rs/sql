// use rdbc_attributes::*;
// use rdbc_orm::{schema::IndexType, Column};

// #[test]
// fn test_table_declare() {
//     #[derive(Debug, Table, Default)]
//     struct Palette {
//         #[col_primary]
//         id: Column<i32>,

//         #[col_index]
//         name: Column<String>,

//         #[col_index(time)]
//         created_time: Column<u64>,

//         #[col_unique(color_opacity)]
//         color: Column<u8>,

//         #[col_name(platette_opacity)]
//         #[col_unique(color_opacity)]
//         opacity: Column<u32>,
//     }

//     assert_eq!(Palette::idx_time().name, "time");

//     assert_eq!(Palette::idx_name().name, "name");

//     assert_eq!(Palette::idx_col_primary().name, "col_primary");

//     assert_eq!(Palette::idx_color_opacity().name, "color_opacity");

//     assert_eq!(Palette::idx_color_opacity().idx_type, IndexType::Unique);

//     assert_eq!(Palette::col_platette_opacity().name, "platette_opacity");
// }
