use proc_macro2::Ident;

use syn::{Data, DataStruct, Fields};

/// extract table field idents
pub fn extract_table_fields(data: &Data) -> anyhow::Result<Vec<Ident>> {
    let fields = match data {
        Data::Struct(DataStruct { fields, .. }) => {
            if let Fields::Named(ref fields_named) = fields {
                let fields: Vec<_> = fields_named
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap().clone())
                    .collect();

                fields
            } else {
                return Err(anyhow::anyhow!("sorry, may it's a complicated struct."));
            }
        }

        _ => {
            return Err(anyhow::anyhow!(
                "sorry, Show is not implemented for union or enum type."
            ));
        }
    };

    Ok(fields)
}
