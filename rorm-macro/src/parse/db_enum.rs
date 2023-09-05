use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use syn::{ItemEnum, Variant, Visibility};

use crate::parse::annotations::NoAnnotations;

pub fn parse_db_enum(tokens: TokenStream) -> darling::Result<ParsedDbEnum> {
    let ItemEnum {
        attrs,
        vis,
        enum_token: _,
        ident,
        generics,
        brace_token: _,
        variants,
    } = syn::parse2(tokens)?;
    let mut errors = darling::Error::accumulator();

    // check absence of #[rorm(..)] attributes
    let _ = errors.handle(NoAnnotations::from_attributes(&attrs));

    // check absence of generics
    if generics.lt_token.is_some() {
        errors.push(darling::Error::unsupported_shape_with_expected(
            "generic struct",
            &"struct without generics",
        ))
    }

    // parse variants
    let mut parsed_variants = Vec::with_capacity(variants.len());
    for variant in variants {
        let Variant {
            attrs,
            ident,
            fields,
            discriminant: _, // TODO maybe warn, that they aren't used?
        } = variant;

        // check absence of #[rorm(..)] attributes
        let _ = errors.handle(NoAnnotations::from_attributes(&attrs));

        // check absence of fields
        if !fields.is_empty() {
            errors.push(
                darling::Error::unsupported_shape("A DbEnum's variants can't contain fields")
                    .with_span(&fields),
            );
        }

        parsed_variants.push(ident);
    }

    errors.finish_with(ParsedDbEnum {
        vis,
        ident,
        variants: parsed_variants,
    })
}

pub struct ParsedDbEnum {
    pub vis: Visibility,
    pub ident: Ident,
    pub variants: Vec<Ident>,
}
