use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type};

use crate::common;

pub fn gin_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_resolvers = &ast.items;

    if let Type::Path(query_type) = *ast.self_ty {
        let root_resolvers = common::parse_tuple_attributes(attrs).map(|mut root_type| {
            let model = root_type.next().expect("a model");
            let schema = root_type.next().expect("a schema");

            let graphql_type = common::gql_struct(&model);
            let singular = Ident::new(model.to_string().to_lowercase().as_ref(), Span::call_site());
            let plural = Ident::new(format!("{}s", singular).as_ref(), Span::call_site());

            quote! {
                fn #singular(context: &Context, id: Uuid) -> FieldResult<#graphql_type> {
                    #schema::table
                        .filter(#schema::id.eq(id))
                        .get_result::<#model>(&context.connection)
                        .map_or_else(
                            |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                            |model| Ok(#graphql_type::from(model.to_owned()))
                        )
                }

                fn #plural(context: &Context, executor: &Executor, ids: Vec<Uuid>, first: Option<i32>, offset: Option<i32>) -> FieldResult<Vec<#graphql_type>> {
                    #schema::table
                        .filter(#schema::id.eq_any(&*ids))
                        .limit(first.unwrap_or(10) as i64)
                        .offset(offset.unwrap_or(0) as i64)
                        .load::<#model>(&context.connection)
                        .map_or_else(
                            |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                            |models| {
                                let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                                #graphql_type::preload_children(&gql_models, &context, &executor.look_ahead());

                                Ok(gql_models)
                            }
                        )
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use gin::Preloadable;

            #[juniper::object(Context = Context)]
            impl #query_type {
                #( #user_defined_resolvers )*
                #( #root_resolvers )*
            }
        };

        return gen.into();
    }

    panic!("Attempted to implement gin_query on invalid query type!");
}
