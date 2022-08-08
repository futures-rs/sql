mod col;
mod der;
mod fields;
mod idx;
mod impl_body;
mod ser;
mod table_name;

use impl_body::impl_body;
use proc_macro::*;
use syn::{parse_macro_input, DeriveInput};

use crate::table_name::table_name;

#[proc_macro_derive(
    Table,
    attributes(
        col_name,
        table_name,
        col_index,
        col_unique,
        col_primary,
        col_primary_autoinc
    )
)]
pub fn rdbc_orm_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        attrs,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let table_name_expanded = table_name(
        &ident,
        &impl_generics,
        &ty_generics,
        where_clause.clone(),
        &attrs,
    )
    .unwrap();

    let impl_body_expanded = impl_body(
        &ident,
        &impl_generics,
        &ty_generics,
        where_clause.clone(),
        &data,
    )
    .unwrap();

    let expanded = quote::quote! {
        #impl_body_expanded


        #table_name_expanded
    };

    #[cfg(debug_assertions)]
    println!("{}", expanded);

    expanded.into()
}
