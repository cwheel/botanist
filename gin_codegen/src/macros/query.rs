use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type};

use crate::common;

pub fn gin_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_resolvers = &ast.items;

    if let Type::Path(query_type) = *ast.self_ty {
        let (query_models, params) = common::parse_ident_attributes(attrs);
        let context_ty = &params
            .get("Context")
            .expect("a context must be specified")
            .ident;
        let primary_key_ty = &params
            .get("PrimaryKey")
            .expect("a primary key type must be specified")
            .ident;

        let root_resolvers = query_models.map(|rich_model| {
            let model = &rich_model.ident;
            let graphql_type = common::gql_struct(&model);
            let model_name = model.to_string();

            let singular = Ident::new(common::lower_first(&model_name).as_ref(), Span::call_site());
            let plural = rich_model.arguments.get("plural").map(|token| token.ident.clone()).unwrap_or(Ident::new(format!("{}s", singular).as_ref(), Span::call_site()));

            quote! {
                fn #singular(context: &#context_ty, id: #primary_key_ty) -> juniper::FieldResult<#graphql_type> {
                    #model::resolve_single(context, id)
                }

                fn #plural(context: &#context_ty, executor: &Executor, ids: Vec<#primary_key_ty>, first: Option<i32>, offset: Option<i32>) -> juniper::FieldResult<Vec<#graphql_type>> {
                    #model::resolve_multiple(context, executor, ids, first, offset)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use gin::internal::{__internal__Preloadable, __internal__RootResolver};

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

pub fn generate_root_resolvers(
    model: &Ident,
    schema: &Ident,
    graphql_type: &Ident,
    context: &Ident,
    id_type: &Type,
) -> proc_macro2::TokenStream {
    quote! {
        impl __internal__RootResolver<#context, #id_type, #graphql_type, juniper::DefaultScalarValue> for #model {
            fn resolve_single(context: &Context, id: #id_type) -> juniper::FieldResult<#graphql_type> {
                    match #model::modify_query(
                        #schema::table
                            .filter(#schema::id.eq(id))
                            .into_boxed(),
                        context
                    ) {
                        Ok(query) => {
                            query
                                .get_result::<#model>(context.get_connection())
                                .map_or_else(
                                    |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                                    |model| Ok(#graphql_type::from(model.to_owned()))
                                )
                        },
                        Err(e) => Err(e)
                    }

            }

            fn resolve_multiple(context: &#context, executor: &juniper::Executor<#context, juniper::DefaultScalarValue>, ids: Vec<#id_type>, first: Option<i32>, offset: Option<i32>) -> juniper::FieldResult<Vec<#graphql_type>> {
                match #model::modify_query(
                    #schema::table
                        .filter(#schema::id.eq_any(&*ids))
                        .limit(first.unwrap_or(10) as i64)
                        .offset(offset.unwrap_or(0) as i64)
                        .into_boxed(),
                    context
                ) {
                    Ok(query) => {
                        query
                            .load::<#model>(context.get_connection())
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
                    },
                    Err(e) => Err(e)
                }
            }
        }
    }
}
