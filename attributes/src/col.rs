use proc_macro2::{Ident, TokenStream};

use quote::{format_ident, ToTokens};
use syn::{Data, DataStruct, Field, Fields, Meta, NestedMeta, Type};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

fn is_rdbc_orm_column(field: &Field) -> Option<TokenStream> {
    match &field.ty {
        Type::Path(path) => {
            if let Some(seg) = path.path.segments.last() {
                if seg.ident == "Column" {
                    match &seg.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            return Some(args.into_token_stream());
                        }

                        _ => {}
                    }
                }
            }

            return None;
        }
        _ => return None,
    }
}

/// Extract table name from table attrs
pub fn extract_column_name(field: &Field) -> anyhow::Result<String> {
    for attr in &field.attrs {
        if let Some(path) = attr.path.get_ident() {
            if path != "col_name" {
                continue;
            }
        }

        let meta: NestedMeta = attr.parse_args()?;

        match meta {
            NestedMeta::Meta(Meta::Path(table_name)) => {
                return Ok(format!("{}", table_name.get_ident().unwrap()));
            }
            _ => {}
        }
    }

    let field_name = field.ident.as_ref().unwrap();

    return Ok(field_name.to_string());
}

/// extract table fields token stream
fn extract_table_columns(data: &Data) -> anyhow::Result<Vec<TokenStream>> {
    let fields = match data {
        Data::Struct(DataStruct { fields, .. }) => {
            if let Fields::Named(ref fields_named) = fields {
                let fields: Vec<_> = fields_named
                    .named
                    .iter()
                    .map(|field| {
                        let field_name = field.ident.as_ref().unwrap();

                        let column_type_path = is_rdbc_orm_column(field).expect(&format!(
                            "table field '{}' type must be rdbc_orm::Column",
                            field_name
                        ));

                        let column_name =  extract_column_name(field).unwrap();

                        let column_fn_name = format_ident!("column_{}", field_name);

                        quote::quote! {
                            pub fn #column_fn_name() -> rdbc_orm::schema::ColumnDef::#column_type_path {
                                rdbc_orm::Column::#column_type_path::column_def_static(#column_name)
                            }
                        }
                    })
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

pub fn expand_column_methods(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    data: &Data,
) -> anyhow::Result<TokenStream> {
    let columns = extract_table_columns(data)?;

    let expanded = quote::quote! {
        impl #impl_generics #struct_name #ty_generics
        #where_clause {
            #(#columns)*
        }
    };

    Ok(expanded)
}
