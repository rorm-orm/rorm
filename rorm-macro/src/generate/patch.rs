use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Type, Visibility};

use crate::parse::patch::ParsedPatch;

pub fn generate_patch(patch: &ParsedPatch) -> TokenStream {
    let ParsedPatch {
        vis,
        ident,
        model,
        fields,
    } = patch;

    let types = fields.iter().map(|field| &field.ty);
    let fields = fields.iter().map(|field| &field.ident);

    let decoder = format_ident!("__{ident}_Decoder");

    quote! {
        ::rorm::generate_patch!(
            vis=#vis,
            patch=#ident,
            model=#model,
            decoder=#decoder,
            #(
                fields=#fields,
                types=#types,
            )*
        );
    }
}

pub fn partially_generate_patch<'a>(
    patch: &Ident,
    model: &impl ToTokens, // Ident or Path
    vis: &Visibility,
    fields: impl Iterator<Item=&'a Ident> + Clone,
    types: impl Iterator<Item=&'a Type> + Clone,
) -> TokenStream {
    let decoder = format_ident!("__{patch}_Decoder");
    quote! {
        ::rorm::generate_patch_partial!(
            vis=#vis,
            patch=#patch,
            model=#model,
            decoder=#decoder,
            #(
                fields=#fields,
                types=#types,
            )*
        );
    }
}
