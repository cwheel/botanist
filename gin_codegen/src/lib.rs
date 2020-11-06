#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod common;
mod macros;

#[proc_macro_attribute]
pub fn gin_object(_: TokenStream, input: TokenStream) -> TokenStream {
    macros::object::gin_object(input)
}

#[proc_macro_attribute]
pub fn gin_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    macros::query::gin_query(attrs, input)
}

#[proc_macro_attribute]
pub fn gin_mutation(attrs: TokenStream, input: TokenStream) -> TokenStream {
    macros::mutation::gin_mutation(attrs, input)
}