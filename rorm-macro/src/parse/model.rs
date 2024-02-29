use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use syn::{parse2, Field, ItemStruct, LitInt, LitStr, Type, Visibility};

use crate::parse::annotations::{Default, Index, OnAction};
use crate::parse::{check_non_generic, get_fields_named};

pub fn parse_model(tokens: TokenStream) -> darling::Result<ParsedModel> {
    let ItemStruct {
        struct_token: _,
        generics,
        fields,
        ident,
        vis,
        attrs,
        semi_token: _,
    } = parse2(tokens)?;
    let mut errors = darling::Error::accumulator();

    // check absence of generics
    errors.handle(check_non_generic(generics));

    // parse struct annotations
    let annos = errors
        .handle(ModelAnnotations::from_attributes(&attrs))
        .unwrap_or_default();

    // parse field annotations
    let mut parsed_fields = Vec::new();
    if let Some(raw_fields) = errors.handle(get_fields_named(fields)) {
        parsed_fields.reserve_exact(raw_fields.named.len());
        for field in raw_fields.named {
            let Field {
                attrs,
                vis,
                ident,
                colon_token: _,
                ty,
            } = field;
            let Some(annos) = errors.handle(ModelFieldAnnotations::from_attributes(&attrs)) else {
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
    }

    errors.finish_with(ParsedModel {
        vis,
        ident,
        annos,
        fields: parsed_fields,
    })
}

pub struct ParsedModel {
    pub vis: Visibility,
    pub ident: Ident,
    pub annos: ModelAnnotations,
    pub fields: Vec<ParsedField>,
}

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct ModelAnnotations {
    pub rename: Option<LitStr>,
    pub insert: Option<Visibility>,
    pub query: Option<Visibility>,
    pub update: Option<Visibility>,
    pub delete: Option<Visibility>,
}

pub struct ParsedField {
    pub vis: Visibility,
    pub ident: Ident,
    pub ty: Type,
    pub annos: ModelFieldAnnotations,
}

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct ModelFieldAnnotations {
    /// `#[rorm(auto_create_time)]`
    pub auto_create_time: bool,

    /// `#[rorm(auto_update_time)]`
    pub auto_update_time: bool,

    /// `#[rorm(auto_increment)]`
    pub auto_increment: bool,

    /// `#[rorm(primary_key)]`
    pub primary_key: bool,

    /// `#[rorm(unique)]`
    pub unique: bool,

    /// `#[rorm(id)]`
    pub id: bool,

    /// `#[rorm(on_delete = "..")]`
    pub on_delete: Option<OnAction>,

    /// `#[rorm(on_update = "..")]`
    pub on_update: Option<OnAction>,

    /// `#[rorm(rename = "..")]`
    pub rename: Option<LitStr>,

    /// `#[rorm(ignore)]`
    //pub ignore: bool,

    /// Parse the `#[rorm(default = ..)]` annotation.
    ///
    /// It accepts a single literal as argument.
    /// Currently the only supported literal types are:
    /// - String
    /// - Integer
    /// - Floating Point Number
    /// - Boolean
    ///
    /// TODO: Figure out how to check the literal's type is compatible with the annotated field's type
    pub default: Option<Default>,

    /// Parse the `#[rorm(max_length = ..)]` annotation.
    ///
    /// It accepts a single integer literal as argument.
    pub max_length: Option<LitInt>,

    /// Parse the `#[rorm(index)]` annotation.
    ///
    /// It accepts four different syntax's:
    /// - `#[rorm(index)]`
    /// - `#[rorm(index())]`
    ///    *(semantically identical to first one)*
    /// - `#[rorm(index(name = <string literal>))]`
    /// - `#[rorm(index(name = <string literal>, priority = <integer literal>))]`
    ///    *(insensitive to argument order)*
    pub index: Option<Index>,
}
