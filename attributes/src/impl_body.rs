use proc_macro2::{Ident, TokenStream};

use syn::{Data, ImplGenerics, TypeGenerics, WhereClause};

use crate::{
    col::expand_column_methods, der::expand_der_methods, idx::expand_idx_methods,
    ser::expand_ser_methods,
};

pub fn impl_body(
    struct_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    data: &Data,
) -> anyhow::Result<TokenStream> {
    let ser = expand_ser_methods(struct_name, impl_generics, ty_generics, where_clause, data)?;

    let der = expand_der_methods(struct_name, impl_generics, ty_generics, where_clause, data)?;

    let columns =
        expand_column_methods(struct_name, impl_generics, ty_generics, where_clause, data)?;

    let idx = expand_idx_methods(struct_name, impl_generics, ty_generics, where_clause, data)?;

    let expanded = quote::quote! {
        #columns

        #ser

        #der

        #idx


    };

    Ok(expanded)
}
