pub trait ColumnValue {
    type ColumnType;

    fn rdbc_type() -> rdbc::ColumnType;
    fn cast_to_rdbc_value(&self) -> anyhow::Result<rdbc::Value>;
    fn cast_from_rdbc_value(value: rdbc::Value) -> anyhow::Result<Self::ColumnType>;
}

impl<T> ColumnValue for Option<T>
where
    T: ColumnValue<ColumnType = T> + Default,
{
    type ColumnType = Option<T>;

    fn cast_from_rdbc_value(value: rdbc::Value) -> anyhow::Result<Self::ColumnType> {
        match value {
            rdbc::Value::Null => Ok(None),
            _ => T::cast_from_rdbc_value(value).map(|v| Some(v)),
        }
    }

    fn cast_to_rdbc_value(&self) -> anyhow::Result<rdbc::Value> {
        if self.is_none() {
            Ok(rdbc::Value::Null)
        } else {
            self.as_ref().unwrap().cast_to_rdbc_value()
        }
    }

    fn rdbc_type() -> rdbc::ColumnType {
        T::rdbc_type()
    }
}

macro_rules! impl_col_value {
    ($t: ty,$rdbc_type: ty, $rdbc_col_type: expr,$rdbc_value: expr,$rdbc_match_expr: pat_param,$rdbc_return_expr: expr) => {
        impl ColumnValue for $t {
            type ColumnType = $t;

            fn cast_from_rdbc_value(value: rdbc::Value) -> anyhow::Result<Self::ColumnType> {
                match value {
                    $rdbc_match_expr => Ok($rdbc_return_expr),

                    _ => Err(anyhow::anyhow!("Invalid rdbc value {:?}", value)),
                }
            }

            fn cast_to_rdbc_value(&self) -> anyhow::Result<rdbc::Value> {
                Ok($rdbc_value(*self as $rdbc_type))
            }

            fn rdbc_type() -> rdbc::ColumnType {
                $rdbc_col_type
            }
        }
    };
}

macro_rules! impl_col_i_values {
    ($($t:ty),*) => {
        $(
            impl_col_value!(
                $t,
                i64,
                rdbc::ColumnType::I64,
                rdbc::Value::I64,
                rdbc::Value::I64(v),
                v as Self::ColumnType
            );
        )*
    };
}

impl_col_i_values!(i8, u8, i16, u16, i32, u32, i64, u64);

impl_col_value!(
    f32,
    f64,
    rdbc::ColumnType::F64,
    rdbc::Value::F64,
    rdbc::Value::F64(v),
    v as Self::ColumnType
);

impl_col_value!(
    f64,
    f64,
    rdbc::ColumnType::F64,
    rdbc::Value::F64,
    rdbc::Value::F64(v),
    v as Self::ColumnType
);

impl ColumnValue for String {
    type ColumnType = String;

    fn cast_from_rdbc_value(value: rdbc::Value) -> anyhow::Result<Self::ColumnType> {
        match value {
            rdbc::Value::String(v) => Ok(v as Self::ColumnType),

            _ => Err(anyhow::anyhow!("Invalid rdbc value {:?}", value)),
        }
    }

    fn cast_to_rdbc_value(&self) -> anyhow::Result<rdbc::Value> {
        Ok(rdbc::Value::String(self.clone()))
    }

    fn rdbc_type() -> rdbc::ColumnType {
        rdbc::ColumnType::String
    }
}

impl ColumnValue for Vec<u8> {
    type ColumnType = Vec<u8>;

    fn cast_from_rdbc_value(value: rdbc::Value) -> anyhow::Result<Self::ColumnType> {
        match value {
            rdbc::Value::Bytes(v) => Ok(v as Self::ColumnType),

            _ => Err(anyhow::anyhow!("Invalid rdbc value {:?}", value)),
        }
    }

    fn cast_to_rdbc_value(&self) -> anyhow::Result<rdbc::Value> {
        Ok(rdbc::Value::Bytes(self.clone()))
    }

    fn rdbc_type() -> rdbc::ColumnType {
        rdbc::ColumnType::Bytes
    }
}
