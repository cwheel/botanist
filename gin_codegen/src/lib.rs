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
            common::TypeRelationship::HasMany(_, _, _) => None,
            common::TypeRelationship::HasOne(_, _, _) => Some(quote! {
                #ident: model.#ident
            }),
            common::TypeRelationship::Field => Some(quote! {
                #ident: model.#ident
            }),
        }
    });

    let resolvers = struct_fields.iter().map(|(field, ty, _)| {
        match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(schema, forign_key, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());

                quote! {
                    pub fn #field(&self, context: &Context) -> Vec<#graphql_type> {
                        let models = #schema::table
                            .filter(#forign_key.eq(&self.id))
                            .load::<#model>(&context.connection)
                            .unwrap();

                        models.iter().map(|model| #graphql_type::from(model.to_owned())).collect::<Vec<#graphql_type>>()
                    }
                }
            },
            common::TypeRelationship::HasOne(_, schema, model) => {
                let graphql_type = Ident::new(format!("{}GQL", model.get_ident().unwrap()).as_ref(), Span::call_site());

                quote! {
                    pub fn #field(&self, context: &Context) -> #graphql_type {
                        let model = #schema::table
                            .filter(#schema::id.eq(&self.#field))
                            .load::<#model>(&context.connection)
                            .unwrap();

                        #graphql_type::from(model.first().unwrap().to_owned())
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
    let attrs = &ast.attrs;
    let gen = quote! {
        // Diesel model struct
        #( #attrs )*
        pub struct #struct_name {
            #( #tokenized_model_fields, )*
        }

        // Juniper struct
        pub struct #gql_struct_name {
            #( #tokenized_graphql_fields, )*
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
    };

    gen.into()
}