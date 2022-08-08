use proc_macro2::{Ident, TokenStream};
use syn::{Attribute, ImplGenerics, Meta, NestedMeta, TypeGenerics, WhereClause};

/// Extract table name from table attrs
pub fn extract_table_name(attrs: &Vec<Attribute>) -> anyhow::Result<Option<String>> {
    for attr in attrs {
        if let Some(path) = attr.path.get_ident() {
            if path != "table_name" {
                continue;
            }
        }

        let meta: NestedMeta = attr.parse_args()?;

        match meta {
            NestedMeta::Meta(Meta::Path(table_name)) => {
                return Ok(Some(format!("{}", table_name.get_ident().unwrap())));
            }
            _ => {}
        }
    }

    return Ok(None);
}

pub fn table_name(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    attrs: &Vec<Attribute>,
) -> anyhow::Result<TokenStream> {
    let table_name = match extract_table_name(attrs)? {
        Some(str) => str,
        None => struct_name.to_string(),
    };

    let expanded = quote::quote! {
        impl #impl_generics #struct_name #ty_generics
        #where_clause {
            pub fn table_name() -> &'static str {
                #table_name
            }
        }
    };

    Ok(expanded.into())
}
