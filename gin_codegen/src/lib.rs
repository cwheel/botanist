#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Type};
use proc_macro2::{Ident, Span};

mod common;

#[proc_macro_attribute]
pub fn gin_object(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let struct_name = &ast.ident;
    let gql_name = struct_name.to_string();
    let gql_struct_name = Ident::new(format!("{}GQL", struct_name).as_ref(), Span::call_site());

    let struct_fields = common::typed_struct_fields_from_ast(&ast);
    let tokenized_model_fields = common::tokenized_struct_fields_from_ast(&ast, |ident, ty| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(_, _, _) => None,
            common::TypeRelationship::HasOne(relationship_ty, _, _) => Some(quote! {
                pub #ident: #relationship_ty
            }),
            common::TypeRelationship::Field => Some(quote! {
                pub #ident: #ty
            }),
        }
    });

    let tokenized_graphql_fields = common::tokenized_struct_fields_from_ast(&ast, |ident, ty| {
        match common::type_relationship(ty) {
            // Relationships retain their underlying type here (some id) to query for them in the resolvers
            common::TypeRelationship::HasMany(_, _, _) => None,
            common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                pub #ident: #relationship_type
            }),
            common::TypeRelationship::Field => Some(quote! {
                pub #ident: #ty
            }),
        }
    });

    let tokenized_from_fields = common::tokenized_struct_fields_from_ast(&ast, |ident, ty| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(_, _, _) => {
                let preloader_field = Ident::new(format!("{}_preloaded", ident).as_ref(), Span::call_site());

                Some(quote! {
                    #preloader_field: RefCell::new(None)
                })
            },
            common::TypeRelationship::HasOne(_, _, _) => {
                let preloader_field = Ident::new(format!("{}_preloaded", ident).as_ref(), Span::call_site());

                Some(quote! {
                    #ident: model.#ident,
                    #preloader_field: RefCell::new(None)
                })
            },
            common::TypeRelationship::Field => Some(quote! {
                #ident: model.#ident
            }),
        }
    });

    let resolvers = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(schema, forign_key, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let preloader_field = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                quote! {
                    pub fn #field(&self, context: &Context) -> Vec<#graphql_type> {
                        if self.#preloader_field.borrow().is_some() {
                            println!("Using existing value!");
                            self.#preloader_field.replace_with(|_| None).unwrap()
                        } else {
                            println!("Not using existing value :(");
                            let models = #schema::table
                                .filter(#forign_key.eq(&self.id))
                                .load::<#model>(&context.connection)
                                .unwrap();

                            let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                            #graphql_type::preload_children(&gql_models, &context);

                            gql_models
                        }
                    }
                }
            },
            common::TypeRelationship::HasOne(_, schema, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let preloader_field = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                quote! {
                    pub fn #field(&self, context: &Context) -> #graphql_type {
                        if self.#preloader_field.borrow().is_some() {
                            println!("Using existing value!");
                            self.#preloader_field.replace_with(|_| None).unwrap()
                        } else {
                            println!("Not using existing value :(");
                            let model = #schema::table
                                .filter(#schema::id.eq(&self.#field))
                                .load::<#model>(&context.connection)
                                .unwrap();

                            #graphql_type::from(model.first().unwrap().to_owned())
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

    let preloader_fields = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, _, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let field_name = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                Some(quote! {
                    #field_name: RefCell<Option<#graphql_type>>
                })
            },
            common::TypeRelationship::HasMany(_, _, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let field_name = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                Some(quote! {
                    #field_name: RefCell<Option<Vec<#graphql_type>>>
                })
            },
            _ => None
        }
    }).filter(|field| field.is_some());

    let id_extractors = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, _, model) => {
                let str_model = model.get_ident().unwrap().to_string();

                Some(quote! {
                    type_to_ids
                        .entry(#str_model)
                        .or_insert(Vec::new())
                        .push(self_model.#field);
                })
            },
            common::TypeRelationship::HasMany(_, _, model) => {
                let str_model = model.get_ident().unwrap().to_string();

                Some(quote! {
                    type_to_ids
                        .entry(#str_model)
                        .or_insert(Vec::new())
                        .push(self_model.id);
                })
            },
            _ => None
        }
    }).filter(|extractor| extractor.is_some());

    let preloaders = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasOne(_, schema, model) => {
                let str_model = model.get_ident().unwrap().to_string();
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let preload_field = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                Some(quote! {
                    {
                        let mut distinct_ids = type_to_ids.get_mut(#str_model).unwrap();
                        distinct_ids.sort();
                        distinct_ids.dedup();

                        let models = #schema::table
                            .filter(#schema::id.eq_any(&*distinct_ids))
                            .load::<#model>(&context.connection)
                            .unwrap();

                        let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                        #graphql_type::preload_children(&gql_models, &context);

                        let distinct_id_to_model: HashMap<&Uuid, #model> = HashMap::from_iter(distinct_ids.iter().zip(models.into_iter()));

                        for self_model in self_models.iter() {
                            let child_model = distinct_id_to_model.get(&self_model.#field).unwrap();
                            self_model.#preload_field.replace_with(|_| Some(#graphql_type::from(child_model.clone())));
                        }
                    }
                })
            },
            common::TypeRelationship::HasMany(schema, forign_key_path, model) => {
                let str_model = model.get_ident().unwrap().to_string();
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());
                let preload_field = Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site());

                let forign_key = forign_key_path.segments.last().unwrap();

                Some(quote! {
                    {
                        let mut forign_key_ids = type_to_ids.get_mut(#str_model).unwrap();
                        forign_key_ids.sort();
                        forign_key_ids.dedup();

                        let models = #schema::table
                            .filter(#schema::#forign_key.eq_any(&*forign_key_ids))
                            .load::<#model>(&context.connection)
                            .unwrap();

                        let gql_models = models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>();
                        #graphql_type::preload_children(&gql_models, &context);

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
                })
            },
            _ => None
        }
    });

    let attrs = &ast.attrs;
    let gen = quote! {
        use gin::Preloadable;
        use std::cell::RefCell;

        // Diesel model struct
        #( #attrs )*
        pub struct #struct_name {
            #( #tokenized_model_fields, )*
        }

        // Juniper struct
        pub struct #gql_struct_name {
            #( #tokenized_graphql_fields, )*
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
            fn preload_children(self_models: &Vec<#gql_struct_name>, context: &Context) {
                use std::collections::HashMap;
                use std::iter::FromIterator;

                let mut type_to_ids: HashMap<&str, Vec<Uuid>> = HashMap::new();

                for self_model in self_models.iter() {
                    #( #id_extractors )*
                }

                #( #preloaders )*
            }
        }
    };

    gen.into()
}