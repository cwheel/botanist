#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod common;
mod macros;

#[proc_macro_attribute]
pub fn botanist_object(attrs: TokenStream, input: TokenStream) -> TokenStream {
    macros::object::botanist_object(attrs, input)
}

#[proc_macro_attribute]
pub fn botanist_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    macros::query::botanist_query(attrs, input)
}

#[proc_macro_attribute]
pub fn botanist_mutation(attrs: TokenStream, input: TokenStream) -> TokenStream {
    macros::mutation::botanist_mutation(attrs, input)
}
