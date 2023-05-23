use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{parse2, Field, Fields, ItemStruct, Path, PathSegment, Type, Visibility};

use crate::parse::annotations::NoAnnotations;

pub fn parse_patch(tokens: TokenStream) -> darling::Result<ParsedPatch> {
    let ItemStruct {
        attrs,
        vis,
        struct_token: _,
        ident,
        generics,
        fields,
        semi_token: _,
    } = parse2(tokens)?;
    let mut errors = darling::Error::accumulator();

    // Parse annotations
    let annos = errors.handle(PatchAnnotations::from_attributes(&attrs));
    let model = annos.map(|annos| annos.model).unwrap_or_else(|| {
        PathSegment {
            ident: format_ident!(""),
            arguments: Default::default(),
        }
        .into()
    });

    // Check absence of generics
    if generics.lt_token.is_some() {
        errors.push(darling::Error::unsupported_shape_with_expected(
            "generic struct",
            &"struct without generics",
        ))
    }

    // Parse fields
    let mut parsed_fields = Vec::new();
    match fields {
        Fields::Named(raw_fields) => {
            parsed_fields.reserve_exact(raw_fields.named.len());
            for field in raw_fields.named {
                let Field {
                    attrs,
                    vis: _,
                    ident,
                    colon_token: _,
                    ty,
                } = field;
                errors.handle(NoAnnotations::from_attributes(&attrs));
                let ident = ident.expect("Fields::Named should contain named fields");
                parsed_fields.push(ParsedPatchField { ident, ty });
            }
        }
        Fields::Unnamed(_) => errors.push(darling::Error::unsupported_shape_with_expected(
            "named tuple",
            &"struct with named fields",
        )),
        Fields::Unit => errors.push(darling::Error::unsupported_shape_with_expected(
            "unit struct",
            &"struct with named fields",
        )),
    }

    errors.finish_with(ParsedPatch {
        vis,
        ident,
        model,
        fields: parsed_fields,
    })
}

pub struct ParsedPatch {
    pub vis: Visibility,
    pub ident: Ident,
    pub model: Path,
    pub fields: Vec<ParsedPatchField>,
}

pub struct ParsedPatchField {
    pub ident: Ident,
    pub ty: Type,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct PatchAnnotations {
    pub model: Path,
}
