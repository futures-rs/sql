use std::marker::PhantomData;

#[derive(Debug)]
/// The schema of col
pub struct ColumnDef<DataType> {
    name: String,
    maker: PhantomData<DataType>,
}

impl<DataType> ColumnDef<DataType> {
    pub fn new(name: &str) -> Self {
        Self {
            maker: PhantomData,
            name: name.to_owned(),
        }
    }

    pub fn col_name(&self) -> &str {
        &self.name
    }
}

macro_rules! rdbc_type_declare {
    ($r_type:ty,$rdbc_type:expr) => {
        impl ColumnDef<$r_type> {
            #[inline]
            pub fn rdbc_type(&self) -> rdbc::ColumnType {
                Self::rdbc_type_static()
            }

            #[inline]
            pub fn rdbc_type_static() -> rdbc::ColumnType {
                $rdbc_type
            }
        }
    };
}

rdbc_type_declare!(u8, rdbc::ColumnType::I64);
rdbc_type_declare!(String, rdbc::ColumnType::String);
rdbc_type_declare!(&str, rdbc::ColumnType::String);
rdbc_type_declare!(f32, rdbc::ColumnType::F64);
rdbc_type_declare!(f64, rdbc::ColumnType::F64);

#[cfg(test)]
mod tests {
    use super::ColumnDef;

    #[test]
    fn test_column_def() {
        assert_eq!(ColumnDef::<u8>::rdbc_type_static(), rdbc::ColumnType::I64);

        assert_eq!(
            ColumnDef::<String>::rdbc_type_static(),
            rdbc::ColumnType::String
        );
    }
}
