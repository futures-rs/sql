use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};

use quote::format_ident;
use syn::{Attribute, Data, DataStruct, Fields, Meta, NestedMeta};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use crate::col::extract_column_name;

/// Extract table name from table attrs
fn extract_idx_name(attrs: &Vec<Attribute>) -> anyhow::Result<Option<(String, TokenStream)>> {
    for attr in attrs {
        let ident = attr.path.get_ident();

        if ident.is_none() {
            continue;
        }

        let ident = ident.unwrap();

        if ident != "col_index" && ident != "col_unique" {
            if ident == "col_primary" {
                return Ok(Some((
                    "col_primary".to_owned(),
                    quote::quote! { rdbc_orm::schema::IndexType::Primary },
                )));
            }

            if ident == "col_primary_autoinc" {
                return Ok(Some((
                    "col_primary".to_owned(),
                    quote::quote! { rdbc_orm::schema::IndexType::PrimaryAutoInc },
                )));
            }

            continue;
        }

        let meta: NestedMeta = attr.parse_args()?;

        match meta {
            NestedMeta::Meta(Meta::Path(table_name)) => {
                if ident == "col_unique" {
                    return Ok(Some((
                        format!("{}", table_name.get_ident().unwrap()),
                        quote::quote! { rdbc_orm::schema::IndexType::Unique },
                    )));
                } else {
                    return Ok(Some((
                        format!("{}", table_name.get_ident().unwrap()),
                        quote::quote! { rdbc_orm::schema::IndexType::Index },
                    )));
                }
            }
            _ => {}
        }
    }

    return Ok(None);
}

/// extract table fields token stream
fn extract_table_idxs(data: &Data) -> anyhow::Result<Vec<TokenStream>> {
    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let mut idxs = HashMap::<String, Vec<String>>::new();
            let mut idx_types = HashMap::<String, TokenStream>::new();

            if let Fields::Named(ref fields_named) = fields {
                for field in &fields_named.named {
                    let field_name = extract_column_name(field)?;

                    match extract_idx_name(&field.attrs).unwrap() {
                        Some((name, index_type)) => {
                            idx_types.insert(name.clone(), index_type);

                            idxs.entry(name).or_insert_with(|| vec![]).push(field_name);
                        }
                        None => continue,
                    }
                }

                return Ok(idxs.iter().map(|(name, field_names)| {
                    let idx_fn_name = format_ident!("idx_{}", name);

                    let idx_type = idx_types.get(name).unwrap();

                    quote::quote! {
                        pub fn #idx_fn_name() -> rdbc_orm::schema::IndexDef {
                            rdbc_orm::schema::IndexDef { name: #name.to_owned(),index_type: #idx_type, for_columns: vec![#(#field_names.to_owned()),*]}
                        }
                    }
                }).collect::<Vec<_>>());
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
}

pub fn expand_idx_methods(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    data: &Data,
) -> anyhow::Result<TokenStream> {
    let columns = extract_table_idxs(data)?;

    let expanded = quote::quote! {
        impl #impl_generics #struct_name #ty_generics
        #where_clause {
            #(#columns)*
        }
    };

    Ok(expanded)
}
