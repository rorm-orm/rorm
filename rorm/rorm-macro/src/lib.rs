//! Implementation of the Model attribute used to implement database things for structs
#![cfg_attr(feature = "unstable", feature(proc_macro_span))]
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Ident, ItemStruct};

/// Create the expression for creating a Option<Source> instance from a span
#[cfg(feature = "unstable")]
fn get_source<T: Spanned>(spanned: &T) -> syn::Expr {
    let span = spanned.span().unwrap();
    syn::parse_str::<syn::Expr>(&format!(
        "Some(::rorm::imr::Source {{
            file: \"{}\".to_string(),
            line: {},
            column: {},
        }})",
        span.source_file().path().display(),
        span.start().line,
        span.start().column,
    ))
    .unwrap()
}
#[cfg(not(feature = "unstable"))]
fn get_source<T: Spanned>(_spanned: &T) -> syn::Expr {
    syn::parse_str::<syn::Expr>("None").unwrap()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn Model(_args: TokenStream, mut input: TokenStream) -> TokenStream {
    let strct = input.clone();
    let strct = parse_macro_input!(strct as ItemStruct);

    let definition_struct = Ident::new(
        &format!("__{}_definition_struct", strct.ident),
        Span::call_site(),
    );
    let definition_instance = Ident::new(
        &format!("__{}_definition_instance", strct.ident),
        Span::call_site(),
    );
    let definition_dyn_object = Ident::new(
        &format!("__{}_definition_dyn_object", strct.ident),
        Span::call_site(),
    );
    let model_name = Literal::string(&strct.ident.to_string());
    let model_source = get_source(&strct);
    let mut model_fields = Vec::new();
    for field in strct.fields.iter() {
        model_fields.push(
            syn::parse_str::<syn::ExprStruct>(&format!(
                "::rorm::imr::Field {{
                    name: \"{}\".to_string(),
                    db_type: <{} as ::rorm::AsDbType>::as_db_type(),
                    annotations: Vec::new(),
                    source: {},
                }}",
                field.ident.as_ref().unwrap(),
                field.ty.to_token_stream(),
                get_source(&field).to_token_stream()
            ))
            .unwrap(),
        );
    }
    input.extend([TokenStream::from({
        quote! {
            #[allow(non_camel_case_types)]
            struct #definition_struct;
            impl ::rorm::ModelDefinition for #definition_struct {
                fn as_imr(&self) -> ::rorm::imr::Model {
                    use ::rorm::imr::*;
                    Model {
                        name: #model_name.to_string(),
                        source: #model_source,
                        fields: vec![ #(#model_fields),* ],
                    }
                }
            }
            #[allow(non_snake_case)]
            static #definition_instance: #definition_struct = #definition_struct;
            #[allow(non_snake_case)]
            #[::linkme::distributed_slice(::rorm::MODELS)]
            static #definition_dyn_object: &'static dyn ::rorm::ModelDefinition = &#definition_instance;
        }
    })]);

    input
}
