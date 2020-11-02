use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use juniper::{LookAheadSelection, DefaultScalarValue, LookAheadMethods};
use syn::{Data, DeriveInput, Ident, Type, Attribute, Lit, Meta, PathArguments, GenericArgument, Path};

pub enum TypeRelationship {
    HasMany(Path, Path, Path),
    HasOne(Path, Path, Path),
    Field
}

enum IterationTypeRelationship {
    HasMany,
    HasOne,
    Field
}

pub fn attribute_from_struct(ast: &DeriveInput, attribute: &str) -> Option<Lit> {
    ast.attrs
        .iter()
        .filter(|attr| attr.path.is_ident(attribute))
        .last()
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => {
                if let Meta::NameValue(attr_meta) = meta {
                    Some(attr_meta.lit)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .unwrap()
}

pub fn attribute_from_struct_as_ident(ast: &DeriveInput, attribute: &str) -> (Option<String>, Option<Ident>) {
    if let Some(attr) = attribute_from_struct(ast, attribute) {
        if let Lit::Str(attr_str) = attr {
            return (Some(attr_str.value()), Some(Ident::new(&attr_str.value(), Span::call_site())))
        }
    }

    (None, None)
}

pub fn typed_struct_fields_from_ast<'a>(ast: &'a DeriveInput) -> Vec<(&'a Ident, &'a Type, &'a Vec<Attribute>)> {
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
        .filter(|maybe_token| maybe_token.is_some())
        .map(|token| token.unwrap())
        .collect::<Vec<proc_macro2::TokenStream>>()
}

pub fn to_camel_case(input: String) -> String {
    let mut first = true;
    input.split("_").map(|token| {
        if first {
            first = false;
            return token.to_owned();
        }

        token.chars().enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap() } else { c.to_lowercase().next().unwrap() })
            .collect::<String>()
    }).collect()
}

pub fn uppercase(input: String) -> String {
    input
        .chars()
        .enumerate()
        .map(|(i, c)|  if i == 0 { c.to_uppercase().next().unwrap() } else { c.to_lowercase().next().unwrap() })
        .collect::<String>()
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
                        let generics = generics.args.iter().map(|generic| {
                            if let GenericArgument::Type(generic_type) = generic {
                                if let Type::Path(path) = generic_type {
                                    // println!("{:?}", path.path);
                                    return path.path.clone();
                                }
                            }

                            panic!("Invalid path in HasOne!");
                        }).collect::<Vec<Path>>();

                        TypeRelationship::HasOne(
                            generics[0].clone(),
                            generics[1].clone(),
                            generics[2].clone(),
                        )
                    },
                    IterationTypeRelationship::HasMany => {
                        let generics = generics.args.iter().map(|generic| {
                            if let GenericArgument::Type(generic_type) = generic {
                                if let Type::Path(path) = generic_type {
                                    // println!("{:?}", path.path);
                                    return path.path.clone();
                                }
                            }

                            panic!("Invalid path in HasMany!");
                        }).collect::<Vec<Path>>();

                        TypeRelationship::HasMany(
                            generics[0].clone(),
                            generics[1].clone(),
                            generics[2].clone(),
                        )
                    },
                }
            }
        }
    }

    TypeRelationship::Field
}