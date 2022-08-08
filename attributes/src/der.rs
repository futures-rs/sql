use proc_macro2::{Ident, TokenStream};

use syn::Data;
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::fields::extract_table_fields;

pub fn expand_der_methods(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    data: &Data,
) -> anyhow::Result<TokenStream> {
    let fields = extract_table_fields(data)?;

    let deserialize = fields
        .iter()
        .map(|field| {
            let lit_str = format!("{}", field);

            quote::quote! {
                der.next(rdbc::Placeholder::Name(#lit_str.to_owned()))?;
                self.#field.orm_deseralize(der)?;

            }
        })
        .collect::<Vec<_>>();

    let expanded = quote::quote! {
        impl #impl_generics #struct_name #ty_generics
        #where_clause {
            pub fn orm_deseralize<D>(&mut self, der: &mut D) -> rdbc_orm::anyhow::Result<()> where D: rdbc_orm::Deserializer {
                #(#deserialize)*

                Ok(())
            }
        }
    };

    Ok(expanded)
}
