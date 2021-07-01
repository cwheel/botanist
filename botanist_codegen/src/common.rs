use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Span, TokenTree};

use syn::{
    Attribute, Data, DeriveInput, GenericArgument, Ident, Lit, Meta, Path, PathArguments, Type,
};

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

pub struct AttributeToken {
    pub ident: Ident,
    pub arguments: HashMap<String, AttributeToken>,
}

#[derive(Debug)]
pub struct InternalAttributeToken {
    pub ident: Ident,
    pub arguments: HashMap<String, Rc<RefCell<InternalAttributeToken>>>,
}

impl InternalAttributeToken {
    pub fn to_attribute_token(token: InternalAttributeToken) -> AttributeToken {
        AttributeToken {
            ident: token.ident.clone(),
            arguments: token
                .arguments
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        InternalAttributeToken::to_attribute_token(
                            Rc::try_unwrap(v).unwrap().into_inner(),
                        ),
                    )
                })
                .collect::<HashMap<String, AttributeToken>>(),
        }
    }
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
                        let mut generics = generics
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

                        let model = generics.remove(2);
                        let schema = generics.remove(1);
                        let id_ty = generics.remove(0);

                        TypeRelationship::HasOne(id_ty, schema, model)
                    }
                    IterationTypeRelationship::HasMany => {
                        let mut generics = generics
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

                        let model = generics.remove(2);
                        let fk_column = generics.remove(1);
                        let schema = generics.remove(0);

                        TypeRelationship::HasMany(schema, fk_column, model)
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

pub fn parse_ident_attributes(
    attrs: TokenStream,
) -> (
    impl Iterator<Item = AttributeToken>,
    HashMap<String, AttributeToken>,
) {
    let (unnamed_values, named_values) = parse_ident_attributes_from_stream(attrs);

    (
        unnamed_values.into_iter().map(|v| {
            InternalAttributeToken::to_attribute_token(Rc::try_unwrap(v).unwrap().into_inner())
        }),
        named_values
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    InternalAttributeToken::to_attribute_token(
                        Rc::try_unwrap(v).unwrap().into_inner(),
                    ),
                )
            })
            .collect::<HashMap<String, AttributeToken>>(),
    )
}

pub fn parse_ident_attributes_from_stream(
    attrs: TokenStream,
) -> (
    impl Iterator<Item = Rc<RefCell<InternalAttributeToken>>>,
    HashMap<String, Rc<RefCell<InternalAttributeToken>>>,
) {
    let idents = proc_macro2::TokenStream::from(attrs).into_iter();

    let mut named_values: HashMap<String, Rc<RefCell<InternalAttributeToken>>> = HashMap::new();
    let mut unnamed_values: Vec<Rc<RefCell<InternalAttributeToken>>> = Vec::new();

    let mut last_token: Option<Rc<RefCell<InternalAttributeToken>>> = None;
    let mut in_expr = false;

    for ident in idents {
        if let TokenTree::Ident(ident) = ident {
            if in_expr {
                let last_attr_token = unnamed_values.pop().unwrap();
                let name = last_attr_token.borrow().ident.to_string();

                named_values.insert(
                    name.clone(),
                    Rc::new(RefCell::new(InternalAttributeToken {
                        ident,
                        arguments: HashMap::new(),
                    })),
                );

                last_token = Some(Rc::clone(named_values.get(&name).unwrap()));
                in_expr = false;
            } else {
                unnamed_values.push(Rc::new(RefCell::new(InternalAttributeToken {
                    ident,
                    arguments: HashMap::new(),
                })));

                last_token = Some(Rc::clone(unnamed_values.last().unwrap()));
            }
        } else if let TokenTree::Group(group) = ident {
            if group.delimiter() == Delimiter::Parenthesis {
                let (tuple_arguments, named_arguments) = parse_ident_attributes_from_stream(group.stream().into());

                if let Some(last_token) = last_token {
                    last_token.borrow_mut().arguments = named_arguments;

                    if in_expr {
                        // If we're in an expression an we encounter a group, treat it as a tuple-like
                        let last_attr_token = unnamed_values.pop().unwrap();
                        let name = last_attr_token.borrow().ident.to_string();

                        // We can only pass InternalAttributeToken's back, so make 'empty' InternalAttributeTokens
                        // The hashmap keys end up being used as the tuple. In an ideal world, arguments becomes
                        // a struct that can support named and unnamed arguments but I don't care enough to fix this
                        // right now.
                        let mut tup_map = HashMap::new();
                        for argument in tuple_arguments {
                            tup_map.insert(argument.borrow().ident.to_string(), Rc::new(RefCell::new(InternalAttributeToken {
                                ident: Ident::new("_", Span::call_site()),
                                arguments: HashMap::new()
                            })));
                        };

                        last_token.borrow_mut().arguments = tup_map;

                        named_values.insert(name, last_token);
                    }
                }

                last_token = None;
            }
        } else if let TokenTree::Punct(character) = ident {
            let raw_char = character.as_char();

            if raw_char == '=' {
                in_expr = true;
            } else if raw_char != ',' {
                panic!("Unexpected punctuation in attribute!");
            }
        } else if let TokenTree::Literal(lit) = ident {
            if !in_expr {
                panic!("Unexpected literal, literals must be named!");
            }

            // Literals are always treated as a string :shrug:
            let str_value = lit.to_string().replace("\"", "");

            let last_attr_token = unnamed_values.pop().unwrap();
            let name = last_attr_token.borrow().ident.to_string();

            named_values.insert(
                name.clone(),
                Rc::new(RefCell::new(InternalAttributeToken {
                    ident: Ident::new(&str_value, Span::call_site()),
                    arguments: HashMap::new(),
                })),
            );

            last_token = Some(Rc::clone(named_values.get(&name).unwrap()));
            in_expr = false;
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

pub fn lower_first(input: &str) -> String {
    let first = input
        .chars()
        .next()
        .unwrap_or(' ')
        .to_string()
        .to_lowercase();
    first + &input[1..]
}
