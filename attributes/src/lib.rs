use proc_macro::*;
use quote::ToTokens;
use syn::*;

#[proc_macro_derive(ORM, attributes(column))]
pub fn rdbc_orm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            if let Fields::Named(ref fields_name) = fields {
                let fields: Vec<_> = fields_name
                    .named
                    .iter()
                    .map(|field| {
                        let field_name = field.ident.as_ref().unwrap();

                        for attr in &field.attrs {
                            print!("field({}) attr({})", field_name, attr.to_token_stream());
                        }

                        quote::quote! {
                            #field_name
                        }
                    })
                    .collect();

                fields
            } else {
                panic!("sorry, may it's a complicated struct.");
            }
        }

        _ => {
            panic!("sorry, Show is not implemented for union or enum type.");
        }
    };

    let struct_name = &input.ident;

    let expanded = quote::quote! {
        impl #impl_generics #struct_name #ty_generics
        #where_clause {
            pub fn orm_seralize<S>(&mut self, ser: &mut S) -> rdbc_orm::anyhow::Result<()> where S: rdbc_orm::Serializer {
                #(
                    self.#fields.orm_seralize(ser)?;
                )*

                Ok(())
            }

            pub fn orm_deseralize<D>(&mut self, de: D) where D: rdbc_orm::Deserializer {
                // #(
                //     self.#fields.orm_deseralize(de);
                // )*
            }
        }
    };

    println!("{}", expanded);

    expanded.into()
}
