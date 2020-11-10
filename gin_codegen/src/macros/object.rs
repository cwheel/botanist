use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::DeriveInput;

use crate::common;
use crate::macros::mutation::{generate_create_mutation, generate_update_mutation, generate_delete_mutation};

pub fn gin_object(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let struct_name = &ast.ident;
    let schema = common::schema_from_struct(&ast).expect("every gin_object to have a table name");

    let gql_name = struct_name.to_string();
    let gql_struct_name = Ident::new(format!("{}GQL", struct_name).as_ref(), Span::call_site());

    let struct_fields = common::typed_struct_fields_from_ast(&ast);

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

    // Fields to implement std::From on the GQL struct for the model
    let tokenized_from_fields =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => {
                    let preload_field = common::preload_field(ident);

                    Some(quote! {
                        #preload_field: RefCell::new(None)
                    })
                }
                common::TypeRelationship::HasOne(_, _, _) => {
                    let preload_field = common::preload_field(ident);

                    Some(quote! {
                        #ident: model.#ident,
                        #preload_field: RefCell::new(None)
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
                    pub fn #field(&self, context: &Context, executor: &Executor, first: Option<i32>, offset: Option<i32>) -> FieldResult<Vec<#graphql_type>> {
                        if self.#preload_field.borrow().is_some() {
                            Ok(self.#preload_field.replace_with(|_| None).unwrap())
                        } else {
                            #schema::table
                                .filter(#forign_key.eq(&self.id))
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
                }
            },
            common::TypeRelationship::HasOne(_, schema, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                quote! {
                    pub fn #field(&self, context: &Context, executor: &Executor) -> FieldResult<#graphql_type> {
                        if self.#preload_field.borrow().is_some() {
                            Ok(self.#preload_field.replace_with(|_| None).unwrap())
                        } else {
                            #schema::table
                                .filter(#schema::id.eq(&self.#field))
                                .get_result::<#model>(&context.connection)
                                .map_or_else(
                                    |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                                    |model| Ok(#graphql_type::from(model.to_owned()))
                                )
                        }
                    }
                }
            },
            common::TypeRelationship::Field => quote! {
                pub fn #field(&self, context: &Context) -> &#ty {
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
                    #preload_field: RefCell<Option<#graphql_type>>
                })
            }
            common::TypeRelationship::HasMany(_, _, model) => {
                let (preload_field, graphql_type) = common::get_type_info(field, &model);

                Some(quote! {
                    #preload_field: RefCell<Option<Vec<#graphql_type>>>
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
                                    .load::<#model>(&context.connection)
                                    .unwrap();

                                let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                                #graphql_type::preload_children(&gql_models, &context, &look_ahead_selection);

                                let distinct_id_to_model: HashMap<&Uuid, #model> = HashMap::from_iter(distinct_ids.iter().zip(models.into_iter()));

                                for self_model in self_models.iter() {
                                    let child_model = distinct_id_to_model.get(&self_model.#field).unwrap();
                                    self_model.#preload_field.replace_with(|_| Some(#graphql_type::from(child_model.clone())));
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
                            let mut forign_key_ids = type_to_ids.get_mut(#str_model).unwrap();
                            forign_key_ids.sort();
                            forign_key_ids.dedup();

                            let first = macro_helpers::int_argument_from_look_ahead(look_ahead, "first", 10);
                            let offset = macro_helpers::int_argument_from_look_ahead(look_ahead, "offset", 0);

                            let models = #schema::table
                                .filter(#schema::#forign_key.eq_any(&*forign_key_ids))
                                .limit(first as i64)
                                .offset(offset as i64)
                                .load::<#model>(&context.connection)
                                .unwrap();

                            let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                            #graphql_type::preload_children(&gql_models, &context, &look_ahead_selection);

                            let mut forign_key_to_models: HashMap<&Uuid, Vec<#model>> = HashMap::new();
                            for model in models.iter() {
                                forign_key_to_models
                                    .entry(&model.#forign_key)
                                    .or_insert(Vec::new())
                                    .push(model.to_owned());
                            }

                            for self_model in self_models.iter() {
                                if let Some(child_models) = forign_key_to_models.get(&self_model.id) {
                                    self_model.#preload_field.replace_with(
                                        |_| Some(
                                            child_models.into_iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>()
                                        )
                                    );
                                }
                            }
                        }
                    }
                })
            },
            _ => None
        }
    });

    // Mutations
    let create_mutation = generate_create_mutation(&ast, &struct_name, &schema, &gql_struct_name);
    let update_mutation = generate_update_mutation(&ast, &struct_name, &schema, &gql_struct_name);
    let delete_mutation = generate_delete_mutation(&struct_name, &schema, &gql_struct_name);

    let attrs = &ast.attrs;
    let gen = quote! {
        use diesel::prelude::*;
        use gin::{CreateMutation, UpdateMutation, DeleteMutation, Preloadable, macro_helpers};
        use std::cell::RefCell;

        use juniper::{Executor, LookAheadSelection, DefaultScalarValue, LookAheadMethods, LookAheadValue, ScalarValue, FieldResult};

        // Diesel model struct
        #( #attrs )*
        pub struct #struct_name {
            #( #tokenized_fields, )*
        }

        // Juniper struct
        pub struct #gql_struct_name {
            #( #tokenized_fields, )*
            #( #preloader_fields, )*
        }

        #[juniper::object(Context = Context, name = #gql_name)]
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

        impl Preloadable<Context, #gql_struct_name> for #gql_struct_name {
            fn preload_children(self_models: &Vec<#gql_struct_name>, context: &Context, look_ahead: &LookAheadSelection<DefaultScalarValue>) {
                use std::collections::HashMap;
                use std::iter::FromIterator;

                let mut type_to_ids: HashMap<&str, Vec<Uuid>> = HashMap::new();

                for self_model in self_models.iter() {
                    #( #id_extractors )*
                }

                #( #preloaders )*
            }
        }

        #create_mutation
        #update_mutation
        #delete_mutation
    };

    gen.into()
}
