//! Implementation of the Model attribute used to implement database things for structs
#![cfg_attr(feature = "unstable", feature(proc_macro_span))]
extern crate proc_macro;
use std::{cell::RefCell, fmt::Display};

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

struct Errors(RefCell<Vec<syn::Error>>);
impl Errors {
    fn new() -> Errors {
        Errors(RefCell::new(Vec::new()))
    }
    fn push(&self, value: syn::Error) {
        self.0.borrow_mut().push(value);
    }
    fn push_new<T: Display>(&self, span: proc_macro2::Span, msg: T) {
        self.push(syn::Error::new(span, msg));
    }
    fn is_empty(&self) -> bool {self.0.borrow().is_empty()}
}
impl IntoIterator for Errors {
    type Item = syn::Error;
    type IntoIter = <Vec<syn::Error> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}

/// Iterate over all "arguments" inside any #[rorm(..)] attribute
///
/// It inforces the rorm attributes to look like function calls (see [syn::Meta::List])
/// as well as excluding literals as their direct arguments (see [syn::NestedMeta::lit])
#[allow(dead_code)]
fn iter_rorm_attributes<'a>(
    attrs: &'a Vec<syn::Attribute>,
    errors: &'a Errors,
) -> impl Iterator<Item = syn::Meta> + 'a {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("rorm"))
        .map(syn::Attribute::parse_meta)
        .map(Result::ok)
        .flatten()
        .map(|meta| match meta {
            syn::Meta::List(syn::MetaList { nested, .. }) => Some(nested.into_iter()),
            _ => {
                errors.push_new(meta.span(), "Attribute should be of shape: `rorm(..)`");
                None
            }
        })
        .flatten()
        .flatten()
        .map(|nested_meta| match nested_meta {
            syn::NestedMeta::Meta(meta) => Some(meta),
            syn::NestedMeta::Lit(_) => {
                errors.push_new(
                    nested_meta.span(),
                    "`rorm(..)` doesn't take literals directly",
                );
                None
            }
        })
        .flatten()
}

/// Used to match over an [syn::Ident] in a similiar syntax as over [&str]s.
///
/// The first argument is the identifier to match.
/// The last argument is a default match arm (`_ => ..`).
/// In between an arbitrary number of match arms can be passed.
///
/// ```ignore
/// let ident = syn::Ident::new("some_identifier", proc_macro2::Span::call_site());
/// match_ident!(ident
///     "foo" => println!("The identifier was 'foo'"),
///     "bar" => println!("The identifier was 'bar'"),
///     _ => println!("The identifier was neither 'foo' nor 'bar'")
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
    let errors = Errors::new();

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
        for meta in iter_rorm_attributes(&field.attrs, &errors) {
            // Get the annotation's identifier.
            // Since one is required for every annotation, error if it is missing.
            let ident = if let Some(ident) = meta.path().get_ident() {
                ident
            } else {
                errors.push_new(meta.path().span(), "expected identifier");
                continue;
            };

            // Get the literal if the attribute is of shape `rorm(<identifier> = <literal>)`
            let arg = match &meta {
                syn::Meta::NameValue(syn::MetaNameValue { lit, .. }) => Some(lit),
                _ => None,
            };

            // The following macros check the "number of arguments" i.e. the shape of the
            // annotation.
            // They unify the error messages and hide the noisy if-else.
            macro_rules! no_arg {
                // Since an annotation with no argument, doesn't require any additional logic,
                // `no_arg!` takes the Annotation variant's name and does everything itself.
                ($name:literal, $variant:literal) => {{
                    if let syn::Meta::Path(_) = meta {
                        annotations
                            .push(concat!("::rorm::imr::Annotation::", $variant).to_string());
                    } else {
                        errors.push_new(
                            meta.span(),
                            concat!($name, " doesn't take any values: #[rorm(", $name, ")]"),
                        );
                    }
                }};
            }
            macro_rules! one_arg {
                // Annotations with arguments need to process these, so the macro just takes an
                // arbitrary block.
                ($name:literal, $then:block) => {{
                    if arg.is_some()
                        $then
                    else {
                        errors.push_new(meta.span(), concat!($name, " expects a value: #[rorm(", $name, " = ..)]"));
                    }
                }};
            }

            match_ident!(ident,
                "auto_create_time" => no_arg!("auto_create_time", "AutoCreateTime"),
                "auto_update_time" => no_arg!("auto_update_time", "AutoUpdateTime"),
                "not_null" => no_arg!("not_null", "NotNull"),
                "primary_key" => no_arg!("primary_key", "PrimaryKey"),
                "unique" => no_arg!("unique", "Unique"),
                "default" => one_arg!("default", {
                    use syn::Lit::*;
                    let arg = arg.unwrap();
                    let (variant, argument) = match arg {
                        Str(string) => {
                            ("String", format!("\"{}\".to_string()", string.value()))
                        }
                        Int(integer) => ("Integer", integer.to_string()),
                        Float(float) => ("Float", format!("{}.into()", float)),
                        Bool(boolean) => ("Boolean", boolean.value.to_string()),
                        _ => {
                            errors.push_new(arg.span(), "unsupported literal");
                            continue;
                        }
                    };
                    annotations.push(format!(
                        "::rorm::imr::Annotation::DefaultValue(::rorm::imr::DefaultValue::{}({}))",
                        variant, argument,
                    ));
                }),
                "max_length" => one_arg!("max_length", {
                    let arg = arg.unwrap();
                    let length = match arg {
                        syn::Lit::Int(integer) => integer.to_string(),
                        _ => {
                            errors.push_new(arg.span(), "max_length expects an integer literal");
                            continue;
                        }
                    };
                    annotations.push(format!("::rorm::imr::Annotation::MaxLength({})", length));
                }),
                "choices" => {
                    if let syn::Meta::List(syn::MetaList { nested, .. }) = &meta {
                        let mut choices = Vec::new();
                        for choice in nested.iter() {
                            let choice = match choice {
                                syn::NestedMeta::Lit(syn::Lit::Str(choice)) => choice,
                                _ => {
                                    errors.push_new(choice.span(), "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]");
                                    continue;
                                }
                            };
                            choices.push(format!("\"{}\".to_string()", choice.value()));
                        }
                        annotations.push(format!(
                            "::rorm::imr::Annotation::Choices(vec![{}])",
                            choices.join(",")
                        ));
                    } else {
                        errors.push_new(meta.span(), "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]");
                    }
                },
                "index" => {
                    match &meta {
                        syn::Meta::Path(_) => {
                            annotations.push("::rorm::imr::Annotation::Index(None)".to_string());
                        },
                        syn::Meta::List(syn::MetaList {nested, ..}) => {
                            let mut name = None;
                            let mut prio = None;
                            for nested_meta in nested.into_iter() {
                                let (path, literal) = if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, lit, ..})) = &nested_meta {
                                    (path.clone(), lit.clone())
                                } else {
                                    errors.push_new(nested_meta.span(), "index expects keyword arguments: #[rorm(index(name = \"...\"))]");
                                    continue;
                                };
                                let ident = if let Some(ident) = path.get_ident() {
                                    ident
                                } else {
                                    errors.push_new(nested_meta.span(), "index expects keyword arguments: #[rorm(index(name = \"...\"))]");
                                    continue;
                                };
                                match_ident!(ident,
                                    "name" => {
                                        if name.is_none() {
                                            match literal {
                                                syn::Lit::Str(literal) => {
                                                    name = Some(literal.value());
                                                },
                                                _ => {
                                                    errors.push_new(literal.span(), "name expects a string literal: #[rorm(index(name = \"...\"))]");
                                                },
                                            }
                                        } else {
                                            errors.push_new(ident.span(), "name has already been set");
                                        }
                                    },
                                    "priority" => {
                                        if prio.is_none() {
                                            match literal {
                                                syn::Lit::Int(literal) => {
                                                        prio = Some(literal);
                                                    },
                                                    _ => {
                                                        errors.push_new(literal.span(), "priority expects a integer literal: #[rorm(index(priority = \"...\"))]");
                                                    },
                                                }
                                            } else {
                                                errors.push_new(ident.span(), "priority has already been set");
                                            }
                                        },
                                        _ => {
                                            errors.push_new(ident.span(), "unknown keyword argument");
                                        }
                                    );
                                }
                                if prio.is_some() && name.is_none() {
                                    errors.push_new(meta.span(), "index also requires a name when a priority is defined");
                                } else {
                                    let inner = if let Some(name) = name {
                                        let prio = if let Some(prio) = prio {
                                            format!("Some({})", prio)
                                        } else {
                                            "None".to_string()
                                        };
                                        format!("Some(::rorm::imr::IndexValue {{name: \"{}\".to_string(), priority: {}}})", name, prio)
                                    } else {
                                        "None".to_string()
                                    };
                                    annotations.push(format!("::rorm::imr::Annotation::Index({})", inner));
                                }
                        },
                        _ => {
                            errors.push_new(meta.span(), "index ether stands on its own or looks like a function call: #[rorm(index)] or #[rorm(index(..))]");
                        }
                    }
                },
                _ => {
                    errors.push_new(ident.span(), "Unknown annotation");
                }
            );
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
    let errors = errors.into_iter().map(syn::Error::into_compile_error);
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

            #(#errors)*
        }
    })
}

/// This attribute is put on your main function.
///
/// When you build with the `rorm-main` feature enabled this attribute will replace your main function.
/// The new main function will simply write all your defined models to `./.models.json`
/// to be further process by the migrator.
///
/// Make sure you have added the feature `rorm-main` to your crate i.e. put the following in your `Cargo.toml`:
/// ```toml
/// [features]
/// rorm-main = []
/// ```
///
/// If you don't like this feature name you can pass the attribute any other name to use instead:
/// ```
/// use rorm::rorm_main;
///
/// #[rorm_main("other-name")]
/// fn main() {}
/// ```
#[proc_macro_attribute]
pub fn rorm_main(args: TokenStream, item: TokenStream) -> TokenStream {
    let errors = Errors::new();

    let main = syn::parse_macro_input!(item as syn::ItemFn);
    let feature = syn::parse::<syn::LitStr>(args).unwrap_or(syn::LitStr::new("rorm-main", Span::call_site()));
    if main.sig.ident != "main" {
        errors.push_new(Span::call_site(), "only allowed on main function");
    }

    (if errors.is_empty() {
        quote! {
            #[cfg(feature = #feature)]
            fn main() -> Result<(), String> {
                let mut file = ::std::fs::File::create(".models.json").map_err(|err| err.to_string())?;
                ::rorm::write_models(&mut file)?;
                return Ok(());
            }
            #[cfg(not(feature = #feature))]
            #main
        }
    } else {
        let errors = errors.into_iter().map(syn::Error::into_compile_error);
        quote! {
            #(#errors)*
            #main
        }
    }).into()
}
