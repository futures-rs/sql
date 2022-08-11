use proc_macro2::{Ident, TokenStream};

use syn::Data;
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::fields::extract_table_fields;

pub fn expand_ser_methods(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    data: &Data,
) -> anyhow::Result<TokenStream> {
    let fields = extract_table_fields(data)?;

    let serialize = fields
        .iter()
        .map(|field| {
            let lit_str = format!("{}", field);

            quote::quote! {
                self.#field.serialize(rdbc::Placeholder::Name(#lit_str.to_owned()), ser)?;
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote::quote! {
        impl #impl_generics rdbc_orm::Serializable for #struct_name #ty_generics
        #where_clause {
            fn serialize<S>(&self, ph: rdbc::Placeholder, ser: &mut S) -> rdbc_orm::anyhow::Result<()> where S: rdbc_orm::Serializer {
                #(#serialize)*

                Ok(())
            }
        }
    };

    Ok(expanded)
}
