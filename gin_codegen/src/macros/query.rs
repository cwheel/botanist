use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type};

use crate::common;

pub fn gin_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_resolvers = &ast.items;

    if let Type::Path(query_type) = *ast.self_ty {
        let (query_models, params) = common::parse_ident_attributes(attrs);
        let context_ty = params.get("Context").expect("a context must be specified");
    
        let root_resolvers = query_models.map(|model| {
            let graphql_type = common::gql_struct(&model);
            let singular = Ident::new(model.to_string().to_lowercase().as_ref(), Span::call_site());
            let plural = Ident::new(format!("{}s", singular).as_ref(), Span::call_site());

            quote! {
                fn #singular(context: &#context_ty, id: Uuid) -> FieldResult<#graphql_type> {
                    #model::resolve_single(context, id)
                }

                fn #plural(context: &#context_ty, executor: &Executor, ids: Vec<Uuid>, first: Option<i32>, offset: Option<i32>) -> FieldResult<Vec<#graphql_type>> {
                    #model::resolve_multiple(context, executor, ids, first, offset)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use gin::{Preloadable, RootResolver};

            #[juniper::object(Context = #context_ty)]
            impl #query_type {
                #( #user_defined_resolvers )*
                #( #root_resolvers )*
            }
        };

        return gen.into();
    }

    panic!("Attempted to implement gin_query on invalid query type!");
}

pub fn generate_root_resolvers(model: &Ident, schema: &Ident, graphql_type: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl RootResolver<Context, Uuid, #graphql_type, DefaultScalarValue> for #model {
            fn resolve_single(context: &Context, id: Uuid) -> FieldResult<#graphql_type> {
                #schema::table
                        .filter(#schema::id.eq(id))
                        .get_result::<#model>(&context.connection)
                        .map_or_else(
                            |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                            |model| Ok(#graphql_type::from(model.to_owned()))
                        )
            }
        
            fn resolve_multiple(context: &Context, executor: &Executor<Context, DefaultScalarValue>, ids: Vec<Uuid>, first: Option<i32>, offset: Option<i32>) -> FieldResult<Vec<#graphql_type>> {
                #schema::table
                    .filter(#schema::id.eq_any(&*ids))
                    .limit(first.unwrap_or(10) as i64)
                    .offset(offset.unwrap_or(0) as i64)
                    .load::<#model>(&context.connection)
                    .map_or_else(
                        |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                        |models| {
                            let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                            let preload_result = #graphql_type::preload_children(&gql_models, &context, &executor.look_ahead());

                            if let Err(preload_err) = preload_result {
                                Err(juniper::FieldError::new(preload_err.to_string(), juniper::Value::null()))
                            } else {
                                Ok(gql_models)
                            }
                        }
                    )
            }
        }
    }
}