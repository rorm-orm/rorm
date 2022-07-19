use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use crate::errors::Errors;
use crate::utils::{get_source, iter_rorm_attributes};

/// Used to match over an [`Ident`] in a similar syntax as over [&str]s.
///
/// The first argument is the identifier to match.
/// The last argument is a default match arm (`_ => ..`).
/// In between an arbitrary number of match arms can be passed.
///
/// ```ignore
/// use syn::Ident;
///
/// let ident = Ident::new("some_identifier", proc_macro2::Span::call_site());
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

pub fn db_enum(enm: TokenStream) -> TokenStream {
    let enm = match syn::parse2::<syn::ItemEnum>(enm) {
        Ok(enm) => enm,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let mut identifiers = Vec::new();
    let mut literals = Vec::new();
    for variant in enm.variants.iter() {
        if variant.fields.is_empty() {
            let ident = variant.ident.clone();
            let literal = syn::LitStr::new(&variant.ident.to_string(), variant.ident.span());
            identifiers.push(ident);
            literals.push(literal);
        } else {
            errors.push_new(variant.span(), "Variants aren't allowed to contain data");
        }
    }
    let enum_name = &enm.ident;

    quote! {
        impl ::rorm::DbEnum for #enum_name {
            fn from_str(string: &str) -> Self {
                use #enum_name::*;
                match string {
                    #(#literals => #identifiers,)*
                    _ => panic!("Unexpected database value"),
                }
            }
            fn to_str(&self) -> &'static str {
                use #enum_name::*;
                match self {
                    #(#identifiers => #literals,)*
                    _ => unreachable!(),
                }
            }
            fn as_choices() -> Vec<String> {
                vec![
                    #(#literals.to_string()),*
                ]
            }

            #errors
        }
    }
}

pub fn model(strct: TokenStream) -> TokenStream {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let span = proc_macro2::Span::call_site();

    let definition_struct = Ident::new(&format!("__{}_definition_struct", strct.ident), span);
    let definition_instance = Ident::new(&format!("__{}_definition_instance", strct.ident), span);
    let definition_dyn_obj = Ident::new(&format!("__{}_definition_dyn_object", strct.ident), span);

    let model_name = syn::LitStr::new(&strct.ident.to_string(), strct.ident.span());
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

            // Parse a simple annotation taking no arguments and simply adding its associated variant
            macro_rules! parse_anno {
                ($name:literal, $variant:literal) => {{
                    if let syn::Meta::Path(_) = meta {
                        let variant = Ident::new($variant, ident.span());
                        annotations.push(quote! {
                            ::rorm::imr::Annotation::#variant
                        });
                    } else {
                        errors.push_new(
                            meta.span(),
                            concat!($name, " doesn't take any values: #[rorm(", $name, ")]"),
                        );
                    }
                }};
            }

            match_ident!(ident,
                "auto_create_time" => parse_anno!("auto_create_time", "AutoCreateTime"),
                "auto_update_time" => parse_anno!("auto_update_time", "AutoUpdateTime"),
                "not_null" => parse_anno!("not_null", "NotNull"),
                "primary_key" => parse_anno!("primary_key", "PrimaryKey"),
                "unique" => parse_anno!("unique", "Unique"),
                "default" => parse_default(&mut annotations, &errors, &meta),
                "max_length" => parse_max_length(&mut annotations, &errors, &meta),
                "choices" => parse_choices(&mut annotations, &errors, &meta),
                "index" => parse_index(&mut annotations, &errors, &meta),
                _ => errors.push_new(ident.span(), "Unknown annotation")
            );
        }
        let field_name = syn::LitStr::new(&field.ident.as_ref().unwrap().to_string(), field.span());
        let field_type = &field.ty;
        let field_source = get_source(&field);
        model_fields.push(quote! {
            {
                let mut annotations = vec![
                    #(#annotations),*
                ];
                let db_type = <#field_type as ::rorm::AsDbType>::as_db_type(&annotations);
                annotations.append(&mut <#field_type as ::rorm::AsDbType>::implicit_annotations());
                ::rorm::model_def::Field {
                    name: #field_name,
                    db_type, annotations,
                    source: #field_source,
                }
            }
        });
    }

    TokenStream::from({
        quote! {
            #[allow(non_camel_case_types)]
            struct #definition_struct;
            impl ::rorm::model_def::ModelDefinition for #definition_struct {
                fn as_rorm(&self) -> ::rorm::model_def::Model {
                    ::rorm::model_def::Model {
                        name: #model_name,
                        source: #model_source,
                        fields: vec![ #(#model_fields),* ],
                    }
                }
            }

            #[allow(non_snake_case)]
            static #definition_instance: #definition_struct = #definition_struct;

            #[allow(non_snake_case)]
            #[::rorm::linkme::distributed_slice(::rorm::model_def::MODELS)]
            #[::rorm::rename_linkme]
            static #definition_dyn_obj: &'static dyn ::rorm::model_def::ModelDefinition = &#definition_instance;

            #errors
        }
    })
}

fn parse_default(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    let arg = match meta {
        syn::Meta::NameValue(syn::MetaNameValue { lit, .. }) => lit,
        _ => {
            errors.push_new(
                meta.span(),
                "default expects a single literal: #[rorm(default = ..)]",
            );
            return;
        }
    };

    let variant = match arg {
        syn::Lit::Str(_) => "String",
        syn::Lit::Int(_) => "Integer",
        syn::Lit::Float(_) => "Float",
        syn::Lit::Bool(_) => "Boolean",
        _ => {
            errors.push_new(arg.span(), "unsupported literal");
            return;
        }
    };

    let variant = Ident::new(variant, arg.span());
    annotations.push(quote! {
        ::rorm::imr::Annotation::DefaultValue(::rorm::imr::DefaultValue::#variant(#arg.into()))
    });
}

fn parse_max_length(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    match meta {
        syn::Meta::NameValue(syn::MetaNameValue {
            lit: syn::Lit::Int(integer),
            ..
        }) => {
            annotations.push(quote! {
                ::rorm::imr::Annotation::MaxLength(#integer)
            });
        }
        _ => {
            errors.push_new(
                meta.span(),
                "max_length expects a single integer literal: #rorm(max_length = 255)",
            );
        }
    }
}

fn parse_choices(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    if let syn::Meta::List(syn::MetaList { nested, .. }) = meta {
        let mut choices = Vec::new();
        for choice in nested.iter() {
            match choice {
                syn::NestedMeta::Lit(syn::Lit::Str(choice)) => {
                    choices.push(choice);
                }
                _ => {
                    errors.push_new(choice.span(), "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]");
                    continue;
                }
            }
        }
        annotations.push(quote! {
            ::rorm::imr::Annotation::Choices(vec![
                #(#choices.to_string()),*
            ])
        });
    } else {
        errors.push_new(
            meta.span(),
            "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]",
        );
    }
}

fn parse_index(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    match &meta {
        syn::Meta::Path(_) => {
            annotations.push(quote! {
                ::rorm::imr::Annotation::Index(None)
            });
        }
        syn::Meta::List(syn::MetaList { nested, .. }) => {
            let mut name = None;
            let mut prio = None;
            for nested_meta in nested.into_iter() {
                let (path, literal) =
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit,
                        ..
                    })) = &nested_meta
                    {
                        (path.clone(), lit.clone())
                    } else {
                        errors.push_new(
                            nested_meta.span(),
                            "index expects keyword arguments: #[rorm(index(name = \"...\"))]",
                        );
                        continue;
                    };
                let ident = if let Some(ident) = path.get_ident() {
                    ident
                } else {
                    errors.push_new(
                        nested_meta.span(),
                        "index expects keyword arguments: #[rorm(index(name = \"...\"))]",
                    );
                    continue;
                };
                match_ident!(ident,
                "name" => {
                    if name.is_none() {
                        match literal {
                            syn::Lit::Str(literal) => {
                                name = Some(literal);
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
                errors.push_new(
                    meta.span(),
                    "index also requires a name when a priority is defined",
                );
            } else {
                let inner = if let Some(name) = name {
                    let prio = if let Some(prio) = prio {
                        quote! { Some(#prio) }
                    } else {
                        quote! { None }
                    };
                    quote! { Some(::rorm::imr::IndexValue { name: #name.to_string(), priority: #prio }) }
                } else {
                    quote! { None }
                };
                annotations.push(quote! { ::rorm::imr::Annotation::Index(#inner) });
            }
        }
        _ => {
            errors.push_new(meta.span(), "index ether stands on its own or looks like a function call: #[rorm(index)] or #[rorm(index(..))]");
        }
    }
}
