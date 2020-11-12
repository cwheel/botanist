use std::collections::HashMap;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};

use syn::{Attribute, Data, DeriveInput, GenericArgument, Ident, Path, PathArguments, Type, Meta, Lit};

pub enum TypeRelationship {
    HasMany(Path, Path, Path),
    HasOne(Path, Path, Path),
    Field,
}

enum IterationTypeRelationship {
    HasMany,
    HasOne,
    Field,
}

pub fn typed_struct_fields_from_ast<'a>(
    ast: &'a DeriveInput,
) -> Vec<(&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    match &ast.data {
        Data::Struct(struct_data) => {
            let mut struct_values: Vec<(&Ident, &Type, &Vec<Attribute>)> = Vec::new();

            for field in struct_data.fields.iter() {
                let ident = &field.ident;

                if let Some(ident) = ident {
                    struct_values.push((ident, &field.ty, &field.attrs));
                }
            }

            struct_values
        }
        Data::Enum(_) => panic!("Expected type Struct, received type Enum!"),
        Data::Union(_) => panic!("Expected type Struct, received type Union!"),
    }
}

pub fn tokenized_struct_fields_from_ast(
    ast: &DeriveInput,
    mut tokenizer: impl FnMut(&Ident, &Type) -> Option<proc_macro2::TokenStream>,
) -> Vec<proc_macro2::TokenStream> {
    typed_struct_fields_from_ast(ast)
        .iter()
        .map(|(field, ty, _)| tokenizer(field, ty))
        .filter_map(|maybe_token| maybe_token)
        .collect::<Vec<proc_macro2::TokenStream>>()
}

pub fn type_relationship(ty: &Type) -> TypeRelationship {
    let mut relation = IterationTypeRelationship::Field;

    if let Type::Path(raw_type) = ty {
        for seg in &raw_type.path.segments {
            if seg.ident == "HasOne" {
                relation = IterationTypeRelationship::HasOne;
            }

            if seg.ident == "HasMany" {
                relation = IterationTypeRelationship::HasMany;
            }

            if let PathArguments::AngleBracketed(generics) = &seg.arguments {
                return match relation {
                    IterationTypeRelationship::Field => TypeRelationship::Field,
                    IterationTypeRelationship::HasOne => {
                        let generics = generics
                            .args
                            .iter()
                            .map(|generic| {
                                if let GenericArgument::Type(generic_type) = generic {
                                    if let Type::Path(path) = generic_type {
                                        return path.path.clone();
                                    }
                                }

                                panic!("Invalid path in HasOne!");
                            })
                            .collect::<Vec<Path>>();

                        TypeRelationship::HasOne(
                            generics[0].clone(),
                            generics[1].clone(),
                            generics[2].clone(),
                        )
                    }
                    IterationTypeRelationship::HasMany => {
                        let generics = generics
                            .args
                            .iter()
                            .map(|generic| {
                                if let GenericArgument::Type(generic_type) = generic {
                                    if let Type::Path(path) = generic_type {
                                        return path.path.clone();
                                    }
                                }

                                panic!("Invalid path in HasMany!");
                            })
                            .collect::<Vec<Path>>();

                        TypeRelationship::HasMany(
                            generics[0].clone(),
                            generics[1].clone(),
                            generics[2].clone(),
                        )
                    }
                };
            }
        }
    }

    TypeRelationship::Field
}

pub fn preload_field(field: &Ident) -> Ident {
    Ident::new(format!("{}_preloaded", field).as_ref(), Span::call_site())
}

pub fn gql_struct(model: &Ident) -> Ident {
    Ident::new(format!("{}GQL", model).as_ref(), Span::call_site())
}

pub fn gql_struct_from_model(model: &Path) -> Ident {
    gql_struct(model.get_ident().unwrap())
}

pub fn get_type_info(field: &Ident, model: &Path) -> (Ident, Ident) {
    (preload_field(field), gql_struct_from_model(model))
}

pub fn parse_tuple_attributes(
    attrs: TokenStream,
) -> impl Iterator<Item = impl Iterator<Item = Ident>> {
    let tuples = proc_macro2::TokenStream::from(attrs).into_iter();

    tuples
        .map(|tuple| {
            if let TokenTree::Group(tuple) = tuple {
                let tuple_group_tokens = tuple.stream().into_iter();

                Some(
                    tuple_group_tokens
                        .map(|tuple_token| {
                            if let TokenTree::Ident(tuple_ident) = tuple_token {
                                Some(tuple_ident)
                            } else {
                                None
                            }
                        })
                        .filter_map(|ident| ident)
                        .into_iter(),
                )
            } else {
                None
            }
        })
        .filter_map(|tuple| tuple)
        .into_iter()
}

pub fn parse_ident_attributes(
    attrs: TokenStream,
) -> (impl Iterator<Item = Ident>, HashMap<String, Ident>) {
    let idents = proc_macro2::TokenStream::from(attrs).into_iter();
    let mut named_values: HashMap<String, Ident> = HashMap::new();
    let mut unnamed_values: Vec<Ident> = Vec::new();

    let mut in_expr = false;

    for ident in idents {
        if let TokenTree::Ident(ident) = ident {
            if in_expr {
                named_values.insert(unnamed_values.pop().unwrap().to_string(), ident);
                in_expr = false;
            } else {
                unnamed_values.push(ident);
            }
        } else if let TokenTree::Punct(character) = ident {
            let raw_char = character.as_char();

            if raw_char == '=' {
                in_expr = true;
            } else if raw_char != ',' {
                panic!("Unexpected punctuation in attribute!");
            }
        } else {
            panic!("Unexpected token in attribute!");
        }
    }

    (unnamed_values.into_iter(), named_values)
}

pub fn schema_from_struct(ast: &DeriveInput) -> Option<Ident> {
    ast.attrs
        .iter()
        .filter(|attr| attr.path.is_ident("table_name"))
        .last()
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => {
                if let Meta::NameValue(attr_meta) = meta {
                    if let Lit::Str(table_name) = attr_meta.lit {
                        Some(Ident::new(&table_name.value(), Span::call_site()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .unwrap()
}