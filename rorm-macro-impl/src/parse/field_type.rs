use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use syn::{Data, DeriveInput, Field, Fields, Visibility};

use crate::parse::model::{ModelFieldAnnotations, ParsedField};

pub fn parse_field_type(tokens: TokenStream) -> darling::Result<ParsedFieldType> {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = syn::parse2(tokens)?;

    let mut errors = darling::Error::accumulator();

    if attrs.iter().any(|attr| attr.path().is_ident("rorm")) {
        errors.push(darling::Error::custom("Attributes are not implemented yet"));
    }

    if !generics.params.is_empty() {
        let (shape, expected) = match &data {
            Data::Struct(_) => ("generic struct", "struct without generics"),
            Data::Enum(_) => ("generic enum", "enum without generics"),
            Data::Union(_) => ("generic union", "union without generics"),
        };
        errors.push(darling::Error::unsupported_shape_with_expected(
            shape, &expected,
        ))
    }

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                // parse field annotations
                let mut parsed_fields = Vec::new();
                parsed_fields.reserve_exact(fields.named.len());
                for field in fields.named {
                    let Field {
                        attrs,
                        vis,
                        mutability: _,
                        ident,
                        colon_token: _,
                        ty,
                    } = field;
                    let Some(annos) = errors.handle(ModelFieldAnnotations::from_attributes(&attrs))
                    else {
                        continue;
                    };
                    let ident = ident.expect("Fields::Named should contain named fields");
                    parsed_fields.push(ParsedField {
                        vis,
                        ident,
                        ty,
                        annos,
                    });
                }
                return errors.finish_with(ParsedFieldType::Struct(ParsedFieldTypeStruct {
                    vis,
                    ident,
                    fields: parsed_fields,
                }));
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() > 1 {
                    errors.push(darling::Error::unsupported_shape_with_expected(
                        "tuple struct with more than one field",
                        &"tuple struct with one field (newtype) or a struct with named fields",
                    ));
                }
                let Some(field) = fields.unnamed.first() else {
                    errors.push(darling::Error::unsupported_shape_with_expected(
                        "tuple struct without field",
                        &"unit struct or a tuple struct with one field (newtype)",
                    ));
                    errors.finish()?;
                    unreachable!();
                };
                errors.push(darling::Error::custom("newtype are not implemented yet"));
            }
            Fields::Unit => {
                return errors.finish_with(ParsedFieldType::Unit(ParsedFieldTypeUnit { ident }))
            }
        },
        Data::Enum(_) => errors.push(darling::Error::custom("enums are not implemented yet")),
        Data::Union(_) => errors.push(darling::Error::unsupported_shape_with_expected(
            "union",
            &"struct or enum",
        )),
    }

    errors.finish()?;
    unreachable!();
}

pub enum ParsedFieldType {
    Struct(ParsedFieldTypeStruct),
    NewType(ParsedFieldTypeNewType),
    Unit(ParsedFieldTypeUnit),
}

pub struct ParsedFieldTypeStruct {
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Vec<ParsedField>,
}

pub struct ParsedFieldTypeUnit {
    pub ident: Ident,
}

pub enum ParsedFieldTypeNewType {}
