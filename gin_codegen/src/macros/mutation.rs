use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{ItemImpl, Type, DeriveInput};

use crate::common;

pub fn gin_mutation(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_mutations = &ast.items;

    if let Type::Path(mutation_type) = *ast.self_ty {
        let mutations = common::parse_tuple_attributes(attrs).map(|mut root_type| {
            let model = root_type.next().expect("a model");

            let graphql_type = common::gql_struct(&model);

            let create_mutation_struct = Ident::new(format!("Create{}Input", model).as_ref(), Span::call_site());
            let create_mutation = Ident::new(format!("create{}", model).as_ref(), Span::call_site());

            let update_mutation_struct = Ident::new(format!("Update{}Input", model).as_ref(), Span::call_site());
            let update_mutation = Ident::new(format!("update{}", model).as_ref(), Span::call_site());

            let delete_mutation = Ident::new(format!("delete{}", model).as_ref(), Span::call_site());

            quote! {
                pub fn #create_mutation(context: &Context, input: #create_mutation_struct) -> #graphql_type {
                    #create_mutation_struct::create(context, input)
                }

                pub fn #update_mutation(context: &Context, input: #update_mutation_struct) -> #graphql_type {
                    #update_mutation_struct::update(context, input)
                }

                pub fn #delete_mutation(context: &Context, id: Uuid) -> #graphql_type {
                    #graphql_type::delete(context, id)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use gin::{CreateMutation, UpdateMutation, DeleteMutation};

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

pub fn generate_create_mutation(ast: &DeriveInput, struct_name: &Ident, schema: &Ident, gql_struct_name: &Ident) -> Option<proc_macro2::TokenStream> {
    let create_mutation_struct = Ident::new(format!("Create{}Input", struct_name).as_ref(), Span::call_site());
    let create_mutation_struct_name = format!("New{}", struct_name);

    let tokenized_create_mutation_fields =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => None,
                common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                    pub #ident: #relationship_type
                }),
                common::TypeRelationship::Field => if ident == "id" { None } else { Some(quote! {
                    pub #ident: #ty
                }) },
            },
        );

    let tokenized_create_mutation_values =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => None,
                common::TypeRelationship::HasOne(_, _, _) => Some(quote! {
                    #schema::#ident.eq(&self_model.#ident)
                }),
                common::TypeRelationship::Field => if ident == "id" { None } else { Some(quote! {
                    #schema::#ident.eq(&self_model.#ident)
                }) },
            },
        );

    if tokenized_create_mutation_fields.is_empty() {
        None
    } else {
        Some(
            quote! {
                #[derive(juniper::GraphQLInputObject)]
                #[graphql(name=#create_mutation_struct_name)]
                pub struct #create_mutation_struct {
                    #( #tokenized_create_mutation_fields, )*
                }

                impl CreateMutation<Context, #create_mutation_struct, #gql_struct_name> for #create_mutation_struct {
                    fn create(context: &Context, self_model: #create_mutation_struct) -> #gql_struct_name {
                        let loaded_model: #struct_name = diesel::insert_into(#schema::table).values(
                            (
                                #( #tokenized_create_mutation_values, )*
                            )
                        ).get_result(&context.connection).unwrap();

                        #gql_struct_name::from(loaded_model)
                    }
                }
            }
        )
    }
}

pub fn generate_update_mutation(ast: &DeriveInput, struct_name: &Ident, schema: &Ident, gql_struct_name: &Ident) -> Option<proc_macro2::TokenStream> {
    let update_mutation_struct = Ident::new(format!("Update{}Input", struct_name).as_ref(), Span::call_site());
    let update_mutation_struct_name = format!("{}Update", struct_name);

    let schema_str = schema.to_string();

    let tokenized_create_mutation_fields =
        common::tokenized_struct_fields_from_ast(
            &ast,
            |ident, ty| match common::type_relationship(ty) {
                common::TypeRelationship::HasMany(_, _, _) => None,
                common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                    pub #ident: Option<#relationship_type>
                }),
                common::TypeRelationship::Field => if ident == "id" { 
                    Some(quote! {
                        pub #ident: #ty
                    })
                 } else {
                    Some(quote! {
                        pub #ident: Option<#ty>
                    })
                },
            },
        );

    // One field just means id
    if tokenized_create_mutation_fields.len() == 1 {
        None
    } else {
        Some(
            quote! {
                #[derive(juniper::GraphQLInputObject, AsChangeset)]
                #[graphql(name=#update_mutation_struct_name)]
                #[table_name = #schema_str]
                pub struct #update_mutation_struct {
                    #( #tokenized_create_mutation_fields, )*
                }

                impl UpdateMutation<Context, #update_mutation_struct, #gql_struct_name> for #update_mutation_struct {
                    fn update(context: &Context, self_model: #update_mutation_struct) -> #gql_struct_name {
                        let loaded_model: #struct_name = diesel::update(
                            #schema::table.filter(#schema::id.eq(&self_model.id))
                        ).set(&self_model).get_result(&context.connection).unwrap();
                        
                        #gql_struct_name::from(loaded_model)
                    }
                }
            }
        )
    }
}

pub fn generate_delete_mutation(struct_name: &Ident, schema: &Ident, gql_struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl DeleteMutation<Context, Uuid, #gql_struct_name> for #gql_struct_name {
            fn delete(context: &Context, id: Uuid) -> #gql_struct_name {
                let delete_result: #struct_name = diesel::delete(
                    #schema::table.filter(#schema::id.eq(id))
                ).get_result(&context.connection).unwrap();
        
                #gql_struct_name::from(delete_result)
            }
        }
    }
}
