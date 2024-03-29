use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{DeriveInput, Type};

use crate::common;
use crate::macros::mutation::{
    generate_create_mutation, generate_delete_mutation, generate_update_mutation,
};
use crate::macros::query::generate_root_resolvers;

pub fn botanist_object(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let struct_name = &ast.ident;
    let schema =
        common::schema_from_struct(&ast).expect("every botanist_object must have a table name");

    let (_, params) = common::parse_ident_attributes(attrs);
    let context_ty = &params
        .get("Context")
        .expect("a context must be specified")
        .ident;
    let modifies_query = params
        .get("ModifiesQuery")
        .map(|token| token.ident.to_string() == "true")
        .unwrap_or(false);

    let gql_name = struct_name.to_string();
    let gql_struct_name = Ident::new(format!("{}GQL", struct_name).as_ref(), Span::call_site());
    let query_ty = Ident::new(format!("{}Query", struct_name).as_ref(), Span::call_site());

    let struct_fields = common::typed_struct_fields_from_ast(&ast);

    // What kind of primary key are we using
    let id_ty = common::typed_struct_fields_from_ast(&ast)
        .iter()
        .filter_map(|(ident, ty, _)| {
            if ident.to_string() == "id" {
                Some(ty.clone())
            } else {
                None
            }
        })
        .collect::<Vec<&Type>>()[0];

    // Fields for the model and GQL structs
    let tokenized_fields =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => None,
                common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                    pub #ident: #relationship_type
                }),
                common::TypeRelationship::Field => Some(quote! {
                    pub #ident: #ty
                }),
            },
        );

    // Fields eligable for full-text search
    let searchable_fields =
        struct_fields
            .iter()
            .filter_map(|(ident, ty, _)| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => None,
                common::TypeRelationship::HasOne(_, _, _) => None,
                common::TypeRelationship::Field => {
                    if let Type::Path(field_type) = ty {
                        if let Some(segment) = field_type.path.segments.first() {
                            if segment.ident.to_string() == "String" {
                                return Some(ident.clone());
                            }
                        }
                    }

                    None
                }
            });

    // Fields to implement std::From on the GQL struct for the model
    let tokenized_from_fields =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => {
                    let preload_field = common::preload_field(ident);

                    Some(quote! {
                        #preload_field: Arc::new(Mutex::new(RefCell::new(None)))
                    })
                }
                common::TypeRelationship::HasOne(_, _, _) => {
                    let preload_field = common::preload_field(ident);

                    Some(quote! {
                        #ident: model.#ident,
                        #preload_field: Arc::new(Mutex::new(RefCell::new(None)))
                    })
                }
                common::TypeRelationship::Field => Some(quote! {
                    #ident: model.#ident
                }),
            },
        );

    // Juniper resolver functions
    let resolvers = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(schema, forign_key, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                quote! {
                    pub fn #field(
                        &self,
                        context: &#context_ty,
                        executor: &Executor<#context_ty, juniper::DefaultScalarValue>,
                        limit: Option<i32>,
                        offset: Option<i32>
                    ) -> juniper::FieldResult<Vec<#graphql_type>> {
                        if let Ok(preload) = self.#preload_field.clone().lock() {
                            if preload.borrow().is_some() {
                                Ok(preload.replace_with(|_| None).unwrap())
                            } else {
                                #schema::table
                                    .filter(#forign_key.eq(&self.id))
                                    .limit(limit.unwrap_or(10) as i64)
                                    .offset(offset.unwrap_or(0) as i64)
                                    .load::<#model>(&context.get_connection())
                                    .map_or_else(
                                        |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                                        |models| {
                                            let gql_models = models.iter().map(
                                                |model| #graphql_type::from(model.to_owned())
                                            ).collect::<Vec<#graphql_type>>();

                                            let preload_result = #graphql_type::preload_children(&
                                                gql_models,
                                                &context,
                                                &executor.look_ahead()
                                            );

                                            if let Err(preload_err) = preload_result {
                                                Err(juniper::FieldError::new(preload_err.to_string(), juniper::Value::null()))
                                            } else {
                                                Ok(gql_models)
                                            }
                                        }
                                    )
                            }
                        } else {
                            Err(juniper::FieldError::new("Failed to acquire lock on preload field!", juniper::Value::null()))
                        }
                    }
                }
            },
            common::TypeRelationship::HasOne(_, schema, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                quote! {
                    pub fn #field(
                        &self,
                        context: &#context_ty,
                        executor: &Executor<#context_ty, juniper::DefaultScalarValue>
                    ) -> juniper::FieldResult<#graphql_type> {
                        if let Ok(preload) = self.#preload_field.clone().lock() {
                            if preload.borrow().is_some() {
                                Ok(preload.replace_with(|_| None).unwrap())
                            } else {
                                #schema::table
                                    .filter(#schema::id.eq(&self.#field))
                                    .get_result::<#model>(&context.get_connection())
                                    .map_or_else(
                                        |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                                        |model| Ok(#graphql_type::from(model.to_owned()))
                                    )
                            }
                        } else {
                            Err(juniper::FieldError::new("Failed to acquire lock on preload field!", juniper::Value::null()))
                        }
                    }
                }
            },
            common::TypeRelationship::Field => quote! {
                pub fn #field(&self, context: &#context_ty) -> &#ty {
                    &self.#field
                }
            },
        }
    });

    // Fields for storing preloaded values
    let preloader_fields = struct_fields
        .iter()
        .map(|(field, ty, _)| match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, _, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                Some(quote! {
                    #preload_field: Arc<Mutex<RefCell<Option<#graphql_type>>>>
                })
            }
            common::TypeRelationship::HasMany(_, _, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                Some(quote! {
                    #preload_field: Arc<Mutex<RefCell<Option<Vec<#graphql_type>>>>>
                })
            }
            _ => None,
        })
        .filter(|field| field.is_some());

    // Model id extractors; extract ids from models for preloading
    let id_extractors = struct_fields
        .iter()
        .map(|(field, ty, _)| match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, _, model) => {
                let str_model = model.get_ident().unwrap().to_string();

                Some(quote! {
                    type_to_ids
                        .entry(#str_model)
                        .or_insert(Vec::new())
                        .push(self_model.#field);
                })
            }
            common::TypeRelationship::HasMany(_, _, model) => {
                let str_model = model.get_ident().unwrap().to_string();

                Some(quote! {
                    type_to_ids
                        .entry(#str_model)
                        .or_insert(Vec::new())
                        .push(self_model.id);
                })
            }
            _ => None,
        })
        .filter(|extractor| extractor.is_some());

    // Logic blocks for relations before their resolvers are explicity invoked
    let preloaders = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, schema, model) => {
                let str_model = model.get_ident().unwrap().to_string();
                let str_field = field.to_string();

                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                Some(quote! {
                    {
                        if let Some(look_ahead_selection) = look_ahead.select_child(#str_field) {
                            if let Some(mut distinct_ids) = type_to_ids.get_mut(#str_model) {
                                distinct_ids.sort();
                                distinct_ids.dedup();

                                let models = #schema::table
                                    .filter(#schema::id.eq_any(&*distinct_ids))
                                    .load::<#model>(&context.get_connection())?;

                                let gql_models = models.into_iter().map(
                                    |model| #graphql_type::from(model)
                                ).collect::<Vec<#graphql_type>>();

                                #graphql_type::preload_children(&gql_models, &context, &look_ahead_selection)?;

                                let distinct_id_to_gql_model: HashMap<&#id_ty, #graphql_type> = HashMap::from_iter(
                                    distinct_ids.iter().zip(gql_models.into_iter())
                                );

                                for self_model in self_models.iter() {
                                    if let Some(child_model) = distinct_id_to_gql_model.get(&self_model.#field) {
                                        if let Ok(preload) = self_model.#preload_field.clone().lock() {
                                            preload.replace_with(|_| Some(child_model.clone()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                })
            },
            common::TypeRelationship::HasMany(schema, forign_key_path, model) => {
                let str_model = model.get_ident().unwrap().to_string();
                let str_field = field.to_string();

                let (preload_field, graphql_type) = common::get_type_info(field, &model);
                let forign_key = forign_key_path.segments.last().unwrap();

                Some(quote! {
                    {
                        if let Some(look_ahead_selection) = look_ahead.select_child(#str_field) {
                            if let Some(mut forign_key_ids) = type_to_ids.get_mut(#str_model) {
                                forign_key_ids.sort();
                                forign_key_ids.dedup();

                                let limit = macro_helpers::int_argument_from_look_ahead(look_ahead, "limit", 10);
                                let offset = macro_helpers::int_argument_from_look_ahead(look_ahead, "offset", 0);

                                let models = #schema::table
                                    .filter(#schema::#forign_key.eq_any(&*forign_key_ids))
                                    .limit(limit as i64)
                                    .offset(offset as i64)
                                    .load::<#model>(&context.get_connection())?;

                                let gql_models = models.into_iter().map(
                                    |model| #graphql_type::from(model)
                                ).collect::<Vec<#graphql_type>>();

                                #graphql_type::preload_children(&gql_models, &context, &look_ahead_selection)?;

                                let mut forign_key_to_models: HashMap<#id_ty, Vec<#graphql_type>> = HashMap::new();

                                for model in gql_models.into_iter() {
                                    forign_key_to_models
                                        .entry(model.#forign_key.clone())
                                        .or_insert(Vec::new())
                                        .push(model);
                                }

                                for self_model in self_models.iter() {
                                    if let Some(child_models) = forign_key_to_models.get(&self_model.id) {
                                        if let Ok(preload) = self_model.#preload_field.clone().lock() {
                                            preload.replace_with(
                                                |_| Some(child_models.clone())
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                })
            },
            _ => None
        }
    });

    // Top level query modifier stub, if ModifiesQuery isn't set, provide a default implementation
    let query_modifier = if modifies_query {
        quote! {}
    } else {
        quote! {
           impl<'a> __internal__DefaultQueryModifier<#query_ty<'a>, #context_ty> for #struct_name {
                fn modify_query(query: #query_ty<'a>, context: &#context_ty) -> Result<#query_ty<'a>, juniper::FieldError> {
                       Ok(query)
                }
           }
        }
    };

    // Mutations
    let create_mutation =
        generate_create_mutation(&ast, &struct_name, &schema, &gql_struct_name, &context_ty);
    let update_mutation =
        generate_update_mutation(&ast, &struct_name, &schema, &gql_struct_name, &context_ty);
    let delete_mutation =
        generate_delete_mutation(&struct_name, &schema, &gql_struct_name, &context_ty, &id_ty);

    // Query Root Resolvers
    let root_resolvers = generate_root_resolvers(
        &struct_name,
        &schema,
        &gql_struct_name,
        &context_ty,
        &id_ty,
        searchable_fields,
    );

    let attrs = &ast.attrs;
    let gen = quote! {
        use diesel::prelude::*;

        use botanist::internal::{
            __internal__CreateMutation,
            __internal__UpdateMutation,
            __internal__DeleteMutation,
            __internal__Preloadable,
            __internal__RootResolver,
            __internal__DefaultQueryModifier,
        };
        use botanist::macro_helpers;
        use botanist::diesel_extensions::prefix_search;
        use botanist::Context as BotanistContext;
        use std::cell::RefCell;
        use std::sync::Mutex;
        use std::sync::Arc;

        use juniper;
        use juniper::Executor;
        use juniper::LookAheadMethods;
        use juniper::DefaultScalarValue;

        // Diesel model struct
        #( #attrs )*
        pub struct #struct_name {
            #( #tokenized_fields, )*
        }

        // Useful query type alias and default modifier (if the user isn't specifying one)
        type #query_ty<'a> = #schema::BoxedQuery<'a, <#context_ty as BotanistContext>::DB>;
        #query_modifier

        // Juniper struct
        #[derive(Clone)]
        pub struct #gql_struct_name {
            #( #tokenized_fields, )*
            #( #preloader_fields, )*
        }

        #[juniper::graphql_object(Context = Context, name = #gql_name, scalar = juniper::DefaultScalarValue)]
        impl #gql_struct_name {
            #( #resolvers )*
        }

        impl From<#struct_name> for #gql_struct_name {
            fn from(model: #struct_name) -> Self {
                Self {
                    #( #tokenized_from_fields, )*
                }
            }
        }

        impl __internal__Preloadable<Context, #gql_struct_name> for #gql_struct_name {
            fn preload_children(
                self_models: &Vec<#gql_struct_name>,
                context: &#context_ty,
                look_ahead: &juniper::LookAheadSelection<juniper::DefaultScalarValue>
            ) -> Result<(), diesel::result::Error> {
                use std::collections::HashMap;
                use std::iter::FromIterator;

                let mut type_to_ids: HashMap<&str, Vec<#id_ty>> = HashMap::new();

                for self_model in self_models.iter() {
                    #( #id_extractors )*
                }

                #( #preloaders )*

                Ok(())
            }
        }

        #create_mutation
        #update_mutation
        #delete_mutation

        #root_resolvers
    };

    gen.into()
}
