use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type};

use crate::common;

pub fn botanist_query(attrs: TokenStream, input: TokenStream) -> TokenStream {
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

        let (root_resolvers, query_types): (Vec<_>, Vec<_>)  = query_models.map(|rich_model| {
            let model = &rich_model.ident;
            let graphql_type = common::gql_struct(&model);
            let model_name = model.to_string();

            let singular = Ident::new(common::lower_first(&model_name).as_ref(), Span::call_site());
            let query_struct_name = Ident::new(format!("{}Query", &model_name).as_ref(), Span::call_site());
            let plural = rich_model.arguments.get("plural").map(|token| token.ident.clone()).unwrap_or(Ident::new(format!("{}s", singular).as_ref(), Span::call_site()));
            let can_fetch_all = rich_model.arguments.get("all").map(|token| token.ident == "true").unwrap_or(false);

            let searchable = rich_model.arguments.get("searchable").map(|token| &token.arguments);
            let searchable_args = searchable.map(
                |args| args.keys().map(
                    |key| {
                        let ki = Ident::new(key, Span::call_site());

                        quote! {
                            pub #ki: Option<String>
                        }
                    }
                ).collect()
            ).unwrap_or(Vec::new());

            let query_struct = {
                if searchable_args.is_empty() {
                    None
                } else {
                    let query_field_inserters = searchable.map(
                        |args| args.keys().map(
                            |key| {
                                let ki = Ident::new(&key, Span::call_site());
        
                                quote! {
                                    if let Some(value) = &self.#ki {
                                        search_query.insert(#key.to_string(), value.clone());
                                    }
                                }
                            }
                        ).collect()
                    ).unwrap_or(Vec::new());

                    Some(quote! {
                        #[derive(juniper::GraphQLInputObject)]
                        pub struct #query_struct_name {
                            #( #searchable_args )*
                        }

                        impl #query_struct_name {
                            pub fn get_query(&self) -> std::collections::HashMap<String, String> {
                                let mut search_query = std::collections::HashMap::new();

                                #( #query_field_inserters )*

                                search_query
                            }
                        }
                    })
                }
            };

            let query_argument = {
                if searchable_args.is_empty() {
                    None
                } else {
                    Some(quote! {
                        query: Option<#query_struct_name>
                    })
                }
            };

            let query_getter = {
                if searchable_args.is_empty() {
                    None
                } else {
                    Some(quote! {
                        query.map(|query| query.get_query())
                    })
                }
            };

            let plural_resolver = if can_fetch_all {
                quote! {
                    fn #plural(
                        context: &#context_ty,
                        executor: &Executor,
                        ids: Option<Vec<#primary_key_ty>>,
                        limit: Option<i32>,
                        offset: Option<i32>,
                        #query_argument,
                    ) -> juniper::FieldResult<Vec<#graphql_type>> {
                        #model::resolve_multiple(context, executor, ids, limit, offset, #query_getter)
                    }
                }
            } else {
                quote! {
                    fn #plural(
                        context: &#context_ty,
                        executor: &Executor,
                        ids: Vec<#primary_key_ty>,
                        limit: Option<i32>,
                        offset: Option<i32>
                    ) -> juniper::FieldResult<Vec<#graphql_type>> {
                        #model::resolve_multiple(context, executor, Some(ids), limit, offset, None)
                    }
                }
            };

            (quote! {
                fn #singular(context: &#context_ty, id: #primary_key_ty) -> juniper::FieldResult<#graphql_type> {
                    #model::resolve_single(context, id)
                }

                #plural_resolver
            }, query_struct)
        })
        .unzip();

        let gen = quote! {
            use botanist::internal::{__internal__Preloadable, __internal__RootResolver};

            #( #query_types )*

            #[juniper::graphql_object(Context = #context_ty, scalar = juniper::DefaultScalarValue)]
            impl #query_type {
                #( #user_defined_resolvers )*
                #( #root_resolvers )*
            }
        };

        return gen.into();
    }

    panic!("Attempted to implement botanist_query on invalid query type!");
}

pub fn generate_root_resolvers<'a, S: Iterator<Item = &'a Ident>>(
    model: &Ident,
    schema: &Ident,
    graphql_type: &Ident,
    context: &Ident,
    id_type: &Type,
    searchable_fields: S,
) -> proc_macro2::TokenStream {
    let searchable = searchable_fields.map(|field| {
        let field_str = field.to_string();

        if cfg!(feature = "postgres_prefix_search") {
            quote! {
                if let Some(search_query) = search_query.get(#field_str) {
                    query = query.or_filter(
                        // Results must contain a prefix match at any position
                        prefix_search::matches(
                            prefix_search::to_tsvector(#schema::#field),
                            prefix_search::to_tsquery(format!("{}:*", search_query))
                        )
                    ).then_order_by(
                        // Results that begin with the prefix are prioritized
                        #schema::#field.ilike(format!("{}%", search_query)).desc()
                    ).then_order_by(
                        // The closer the prefix is to the start of the string, the higher it ranks
                        prefix_search::position(#schema::#field, search_query.clone()).asc()
                    );
                }
            }
        } else {
            quote! {
                if let Some(search_query) = search_query.get(#field_str) {
                    query = query.or_filter(#schema::#field.ilike(format!("%{}%", search_query)));
                }
            }
        }
    });

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
                            let connection = context.get_connection();

                            query
                                .get_result::<#model>(&connection)
                                .map_or_else(
                                    |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                                    |model| Ok(#graphql_type::from(model.to_owned()))
                                )
                        },
                        Err(e) => Err(e)
                    }

            }

            fn resolve_multiple(
                context: &#context,
                executor: &juniper::Executor<#context, juniper::DefaultScalarValue>,
                ids: Option<Vec<#id_type>>,
                limit: Option<i32>,
                offset: Option<i32>,
                search_query: Option<std::collections::HashMap<String, String>>
            ) -> juniper::FieldResult<Vec<#graphql_type>> {
                let mut query = if let Some(ids) = ids {
                    #schema::table.filter(#schema::id.eq_any(ids))
                        .limit(limit.unwrap_or(10) as i64)
                        .offset(offset.unwrap_or(0) as i64)
                        .into_boxed()
                } else {
                    #schema::table.select(#schema::all_columns)
                        .limit(limit.unwrap_or(10) as i64)
                        .offset(offset.unwrap_or(0) as i64)
                        .into_boxed()
                };

                if let Some(search_query) = search_query {
                    #( #searchable )*
                }

                match #model::modify_query(query, context) {
                    Ok(query) => {
                        let connection = context.get_connection();

                        query
                            .load::<#model>(&connection)
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
