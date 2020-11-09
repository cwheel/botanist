use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type};

use crate::common;

pub fn gin_mutation(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_mutations = &ast.items;

    if let Type::Path(mutation_type) = *ast.self_ty {
        let mutations = common::parse_tuple_attributes(attrs).map(|mut root_type| {
            let model = root_type.next().expect("a model");
            let schema = root_type.next().expect("a schema");

            let graphql_type = common::gql_struct(&model);

            let create_mutation_struct = Ident::new(format!("Create{}Input", model).as_ref(), Span::call_site());
            let create_mutation = Ident::new(format!("create{}", model).as_ref(), Span::call_site());

            quote! {
                pub fn #create_mutation(context: &Context, input: #create_mutation_struct) -> #graphql_type {
                    #create_mutation_struct::create(context, input)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use gin::CreateMutation;

            #[juniper::object(Context = Context)]
            impl #mutation_type {
                #( #user_defined_mutations )*
                #( #mutations )*
            }
        };

        return gen.into();
    }

    panic!("Attempted to implement gin_mutation on invalid mutation type!");
}
