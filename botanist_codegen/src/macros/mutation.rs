use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{DeriveInput, ItemImpl, Type};

use crate::common;

pub fn botanist_mutation(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast: ItemImpl = syn::parse(input).unwrap();
    let user_defined_mutations = &ast.items;

    if let Type::Path(mutation_type) = *ast.self_ty {
        let (mutation_models, params) = common::parse_ident_attributes(attrs);
        let context_ty = &params
            .get("Context")
            .expect("a context must be specified")
            .ident;
        let primary_key_ty = &params
            .get("PrimaryKey")
            .expect("a primary key type must be specified")
            .ident;

        let mutations = mutation_models.map(|token| {
            let model = &token.ident;
            let graphql_type = common::gql_struct(&model);

            let create_mutation_struct = Ident::new(format!("Create{}Input", model).as_ref(), Span::call_site());
            let create_mutation = Ident::new(format!("create{}", model).as_ref(), Span::call_site());

            let update_mutation_struct = Ident::new(format!("Update{}Input", model).as_ref(), Span::call_site());
            let update_mutation = Ident::new(format!("update{}", model).as_ref(), Span::call_site());

            let delete_mutation = Ident::new(format!("delete{}", model).as_ref(), Span::call_site());

            quote! {
                pub fn #create_mutation(context: &#context_ty, input: #create_mutation_struct) -> juniper::FieldResult<#graphql_type> {
                    #create_mutation_struct::create(context, input)
                }

                pub fn #update_mutation(context: &#context_ty, input: #update_mutation_struct) -> juniper::FieldResult<#graphql_type> {
                    #update_mutation_struct::update(context, input)
                }

                pub fn #delete_mutation(context: &#context_ty, id: #primary_key_ty) -> juniper::FieldResult<#graphql_type> {
                    #graphql_type::delete(context, id)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

        let gen = quote! {
            use botanist::internal::{__internal__CreateMutation, __internal__UpdateMutation, __internal__DeleteMutation};

            #[juniper::graphql_object(Context = #context_ty, scalar = juniper::DefaultScalarValue)]
            impl #mutation_type {
                #( #user_defined_mutations )*
                #( #mutations )*
            }
        };

        return gen.into();
    }

    panic!("Attempted to implement botanist_mutation on invalid mutation type!");
}

pub fn generate_create_mutation(
    ast: &DeriveInput,
    struct_name: &Ident,
    schema: &Ident,
    gql_struct_name: &Ident,
    context: &Ident,
) -> Option<proc_macro2::TokenStream> {
    let create_mutation_struct = Ident::new(
        format!("Create{}Input", struct_name).as_ref(),
        Span::call_site(),
    );
    let create_mutation_struct_name = format!("New{}", struct_name);

    let schema_str = schema.to_string();

    let tokenized_create_mutation_fields = common::tokenized_struct_fields_from_ast(
        &ast,
        |ident, ty| match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(_, _, _) => None,
            common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                pub #ident: #relationship_type
            }),
            common::TypeRelationship::Field => {
                if ident == "id" {
                    None
                } else {
                    Some(quote! {
                        pub #ident: #ty
                    })
                }
            }
        },
    );

    if tokenized_create_mutation_fields.is_empty() {
        None
    } else {
        Some(quote! {
            #[derive(juniper::GraphQLInputObject, Insertable)]
            #[graphql(name=#create_mutation_struct_name)]
            #[table_name = #schema_str]
            pub struct #create_mutation_struct {
                #( #tokenized_create_mutation_fields, )*
            }

            impl __internal__CreateMutation<Context, #create_mutation_struct, #gql_struct_name> for #create_mutation_struct {
                fn create(context: &#context, self_model: #create_mutation_struct) -> juniper::FieldResult<#gql_struct_name> {
                    let connection = context.get_connection();

                    diesel::insert_into(#schema::table)
                        .values(&self_model)
                        .get_result(&connection)
                        .map_or_else(
                            |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                            |create_result: #struct_name| Ok(#gql_struct_name::from(create_result))
                        )
                }
            }
        })
    }
}

pub fn generate_update_mutation(
    ast: &DeriveInput,
    struct_name: &Ident,
    schema: &Ident,
    gql_struct_name: &Ident,
    context: &Ident,
) -> Option<proc_macro2::TokenStream> {
    let update_mutation_struct = Ident::new(
        format!("Update{}Input", struct_name).as_ref(),
        Span::call_site(),
    );
    let update_mutation_struct_name = format!("{}Update", struct_name);

    let schema_str = schema.to_string();

    let tokenized_create_mutation_fields = common::tokenized_struct_fields_from_ast(
        &ast,
        |ident, ty| match common::type_relationship(ty) {
            common::TypeRelationship::HasMany(_, _, _) => None,
            common::TypeRelationship::HasOne(relationship_type, _, _) => Some(quote! {
                pub #ident: Option<#relationship_type>
            }),
            common::TypeRelationship::Field => {
                if ident == "id" {
                    Some(quote! {
                        pub #ident: #ty
                    })
                } else {
                    Some(quote! {
                        pub #ident: Option<#ty>
                    })
                }
            }
        },
    );

    // One field just means id
    if tokenized_create_mutation_fields.len() == 1 {
        None
    } else {
        Some(quote! {
            #[derive(juniper::GraphQLInputObject, AsChangeset)]
            #[graphql(name=#update_mutation_struct_name)]
            #[table_name = #schema_str]
            pub struct #update_mutation_struct {
                #( #tokenized_create_mutation_fields, )*
            }

            impl __internal__UpdateMutation<#context, #update_mutation_struct, #gql_struct_name> for #update_mutation_struct {
                fn update(context: &#context, self_model: #update_mutation_struct) -> juniper::FieldResult<#gql_struct_name> {
                    let connection = context.get_connection();

                    diesel::update(
                        #schema::table.filter(#schema::id.eq(&self_model.id))
                    )
                    .set(&self_model)
                    .get_result(&connection)
                    .map_or_else(
                        |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                        |update_result: #struct_name| Ok(#gql_struct_name::from(update_result))
                    )
                }
            }
        })
    }
}

pub fn generate_delete_mutation(
    struct_name: &Ident,
    schema: &Ident,
    gql_struct_name: &Ident,
    context: &Ident,
    id_type: &Type,
) -> proc_macro2::TokenStream {
    quote! {
        impl __internal__DeleteMutation<#context, #id_type, #gql_struct_name> for #gql_struct_name {
            fn delete(context: &#context, id: #id_type) -> juniper::FieldResult<#gql_struct_name> {
                let connection = context.get_connection();

                diesel::delete(
                    #schema::table.filter(#schema::id.eq(id))
                )
                .get_result(&connection)
                .map_or_else(
                    |error| Err(juniper::FieldError::new(error.to_string(), juniper::Value::null())),
                    |delete_result: #struct_name| Ok(#gql_struct_name::from(delete_result))
                )
            }
        }
    }
}
