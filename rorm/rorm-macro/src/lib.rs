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

/// Iterate over all "arguments" inside any #[rorm(..)] attribute
///
/// It inforces the rorm attributes to look like function calls (see [syn::Meta::List])
/// as well as excluding literals as their direct arguments (see [syn::NestedMeta::lit])
#[allow(dead_code)]
fn iter_rorm_attributes<'a>(
    attrs: &'a Vec<syn::Attribute>,
) -> impl Iterator<Item = syn::Meta> + 'a {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("rorm"))
        .map(syn::Attribute::parse_meta)
        .map(Result::ok)
        .flatten()
        .map(|meta| match meta {
            syn::Meta::List(syn::MetaList { nested, .. }) => nested.into_iter(),
            _ => panic!("Attribute should be of shape: `rorm(..)`"),
        })
        .flatten()
        .map(|nested_meta| match nested_meta {
            syn::NestedMeta::Meta(meta) => meta,
            syn::NestedMeta::Lit(_) => panic!("`rorm(..)` doesn't take literals"),
        })
}

/// Used to match over an [syn::Ident] in a similiar syntax as over [&str]s.
///
/// The first argument is the identifier to match.
/// The last argument is a default match arm (`_ => ..`).
/// In between an arbitrary number of match arms can be passed.
///
/// ```
/// let ident = syn::Ident::new("some_identifier", proc_macro2::Span::call_site());
/// match_ident!(ident
///     "foo" => println!("The identifier was 'foo'"),
///     "bar" => println!("The identifier was 'bar'");
///     _ => println!("The identifier was neither 'foo' nor 'bar'");
/// );
/// ```
///
/// Since [proc_macro2] hides the underlying implementation, it is impossible to actually match
/// over the underlying [&str]. So this macro expands into a lot of `if`s and `else`s.
macro_rules! match_ident {
    ($ident:expr, $( $name:literal => $block:expr ),+, _ => $default:expr) => {
        {
            let ident = $ident;
            $(
                if ident == $name {
                    $block
                } else
            )+
            { $default }
        }
    };
}

/// This attribute is used to turn a struct into a database model.
///
/// ```
/// use rorm::Model;
///
/// #[derive(Model)]
/// struct User {
///     #[rorm(primary_key)]
///     id: i32,
///     #[rorm(max_length = 255, unique)]
///     username: String,
///     #[rorm(max_length = 255)]
///     password: String,
///     #[rorm(default = false)]
///     admin: bool,
///     age: u8,
///     #[rorm(choices("m", "f", "d"))]
///     gender: String,
/// }
/// ```
#[allow(non_snake_case)]
#[proc_macro_derive(Model, attributes(rorm))]
pub fn Model(strct: TokenStream) -> TokenStream {
    let strct = parse_macro_input!(strct as ItemStruct);
    let span = Span::call_site();

    let definition_struct = Ident::new(&format!("__{}_definition_struct", strct.ident), span);
    let definition_instance = Ident::new(&format!("__{}_definition_instance", strct.ident), span);
    let definition_dyn_obj = Ident::new(&format!("__{}_definition_dyn_object", strct.ident), span);

    let model_name = Literal::string(&strct.ident.to_string());
    let model_source = get_source(&strct);
    let mut model_fields = Vec::new();
    for field in strct.fields.iter() {
        let mut annotations = Vec::new();
        for meta in iter_rorm_attributes(&field.attrs) {
            match meta {
                syn::Meta::Path(path) => {
                    annotations.push(
                        match_ident!(path.get_ident().expect("Malformed attribute argument"),
                            "auto_create_time" => "::rorm::imr::Annotation::AutoCreateTime",
                            "auto_update_time" => "::rorm::imr::Annotation::AutoUpdateTime",
                            "not_null" => "::rorm::imr::Annotation::NotNull",
                            "primary_key" => "::rorm::imr::Annotation::PrimaryKey",
                            "unique" => "::rorm::imr::Annotation::Unique",
                            "index" => "::rorm::imr::Annotation::Index(None)", // TODO implement
                                                                               // composite index
                            _ => panic!("Unknown annotation")
                        )
                        .to_string(),
                    );
                }
                syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
                    use syn::Lit::*;
                    match_ident!(path.get_ident().expect("Malformed attribute argument"),
                        "default" => {
                            let (variant, argument) = match lit {
                                Str(string) => {
                                    ("String", format!("\"{}\".to_string()", string.value()))
                                }
                                Int(integer) => ("Integer", integer.to_string()),
                                Float(float) => ("Float", float.to_string()),
                                Bool(boolean) => ("Boolean", boolean.value.to_string()),
                                _ => panic!("Unsupported default literal"),
                            };
                            annotations.push(format!(
                                "::rorm::imr::Annotation::DefaultValue(::rorm::imr::DefaultValue::{}({}))",
                                variant, argument,
                            ));
                        },
                        "max_length" => {
                            let length = match lit {
                                Int(integer) => integer.to_string(),
                                _ => panic!("max_length does only support integers"),
                            };
                            annotations.push(format!("::rorm::imr::Annotation::MaxLength({})", length));
                        },
                        _ => panic!("Unknown annotation")
                    );
                }
                syn::Meta::List(syn::MetaList { path, nested, .. }) => {
                    match_ident!(path.get_ident().expect("Malformed attribute argument"),
                        "choices" => {
                            let choices: Vec<String> = nested
                                .into_iter()
                                .map(|nested_meta| match nested_meta {
                                    syn::NestedMeta::Meta(_) => {
                                        panic!("Only literals are supported as choices")
                                    }
                                    syn::NestedMeta::Lit(lit) => lit,
                                })
                                .map(|lit| match lit {
                                    syn::Lit::Str(string) => {
                                        format!("\"{}\".to_string()", string.value())
                                    }
                                    _ => {
                                        panic!("Curently only string literals are supported as choices")
                                    }
                                })
                                .collect();
                            annotations.push(format!(
                                "::rorm::imr::Annotation::Choices(vec![{}])",
                                choices.join(",")
                            ));
                        },
                        _ => panic!("Unknown annotation")
                    );
                }
            }
        }
        model_fields.push(
            syn::parse_str::<syn::ExprStruct>(&format!(
                "::rorm::imr::Field {{
                    name: \"{}\".to_string(),
                    db_type: <{} as ::rorm::AsDbType>::as_db_type(),
                    annotations: vec![{}],
                    source_defined_at: {},
                }}",
                field.ident.as_ref().unwrap(),
                field.ty.to_token_stream(),
                annotations.join(", "),
                get_source(&field).to_token_stream()
            ))
            .unwrap(),
        );
    }
    TokenStream::from({
        quote! {
            #[allow(non_camel_case_types)]
            struct #definition_struct;
            impl ::rorm::ModelDefinition for #definition_struct {
                fn as_imr(&self) -> ::rorm::imr::Model {
                    use ::rorm::imr::*;
                    Model {
                        name: #model_name.to_string(),
                        source_defined_at: #model_source,
                        fields: vec![ #(#model_fields),* ],
                    }
                }
            }
            #[allow(non_snake_case)]
            static #definition_instance: #definition_struct = #definition_struct;
            #[allow(non_snake_case)]
            #[::linkme::distributed_slice(::rorm::MODELS)]
            static #definition_dyn_obj: &'static dyn ::rorm::ModelDefinition = &#definition_instance;
        }
    })
}
