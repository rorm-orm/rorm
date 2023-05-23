use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{parse2, Field, ItemStruct, Path, PathSegment, Type, Visibility};

use crate::parse::annotations::NoAnnotations;
use crate::parse::{check_non_generic, get_fields_named};

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
    errors.handle(check_non_generic(generics));

    // Parse fields
    let mut parsed_fields = Vec::new();
    if let Some(raw_fields) = errors.handle(get_fields_named(fields)) {
        parsed_fields.reserve_exact(raw_fields.named.len());
        for field in raw_fields.named {
            let Field {
                attrs,
                vis: _,
                ident,
                colon_token: _,
                ty,
            } = field;

            // Patch fields don't accept annotations
            errors.handle(NoAnnotations::from_attributes(&attrs));

            let ident = ident.expect("Fields::Named should contain named fields");
            parsed_fields.push(ParsedPatchField { ident, ty });
        }
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
