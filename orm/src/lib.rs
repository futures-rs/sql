#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod serde;

pub use crate::serde::*;

mod crud;

pub use crud::*;

mod schema;

pub use schema::*;
