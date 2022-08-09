use std::marker::PhantomData;

#[derive(Debug)]
/// The schema of col
pub struct ColumnDef<DataType> {
    pub name: &'static str,
    maker: PhantomData<DataType>,
}

impl<DataType> ColumnDef<DataType> {
    pub fn new(name: &'static str) -> Self {
        Self {
            maker: PhantomData,
            name,
        }
    }
}
