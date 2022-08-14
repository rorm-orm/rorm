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
        impl ::rorm::model::DbEnum for #enum_name {
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

    let mut model_name = strct.ident.to_string();
    model_name.make_ascii_lowercase();
    let model_name = syn::LitStr::new(&model_name, strct.ident.span());
    let model_source = get_source(&strct);
    let mut model_fields = Vec::new();
    let mut field_idents = Vec::new();
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
                "primary_key" => parse_anno!("primary_key", "PrimaryKey"),
                "unique" => parse_anno!("unique", "Unique"),
                "autoincrement" => parse_anno!("autoincrement", "AutoIncrement"),
                "default" => parse_default(&mut annotations, &errors, &meta),
                "max_length" => parse_max_length(&mut annotations, &errors, &meta),
                "choices" => parse_choices(&mut annotations, &errors, &meta),
                "index" => parse_index(&mut annotations, &errors, &meta),
                _ => errors.push_new(ident.span(), "Unknown annotation")
            );
        }
        let mut field_name = field.ident.as_ref().unwrap().to_string();
        field_name.make_ascii_lowercase();
        let field_name = syn::LitStr::new(&field_name, field.span());
        let field_type = &field.ty;
        let field_type = quote! { <#field_type as ::rorm::model::AsDbType> };
        let field_source = get_source(&field);
        model_fields.push(quote! {
            {
                let mut annotations = vec![
                    #(#annotations),*
                ];
                let db_type = #field_type::as_db_type(&annotations);
                annotations.append(&mut #field_type::implicit_annotations());
                ::rorm::model::Field {
                    name: #field_name,
                    db_type, annotations,
                    nullable: #field_type::is_nullable(),
                    source: #field_source,
                }
            }
        });
        field_idents.push(field.ident.clone());
    }

    // Empty struct to implement ModelDefinition on
    let definition_getter_struct =
        Ident::new(&format!("__{}_definition_struct", strct.ident), span);
    // Instance of said empty struct
    let definition_getter_instance =
        Ident::new(&format!("__{}_definition_instance", strct.ident), span);
    // Trait object from said instance
    let definition_getter_dyn_obj =
        Ident::new(&format!("__{}_definition_dyn_object", strct.ident), span);

    // Enum containing all model's fields
    let fields_enum = Ident::new(&format!("__{}_Fields", strct.ident), span);

    let strct_ident = strct.ident;
    TokenStream::from({
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            pub enum #fields_enum {
                #(#field_idents),*
            }
            impl ::rorm::model::Model for #strct_ident {
                type Fields = #fields_enum;
            }

            #[allow(non_camel_case_types)]
            struct #definition_getter_struct;
            impl ::rorm::model::GetModelDefinition for #definition_getter_struct {
                fn as_rorm(&self) -> ::rorm::model::ModelDefinition {
                    ::rorm::model::ModelDefinition {
                        name: #model_name,
                        source: #model_source,
                        fields: vec![ #(#model_fields),* ],
                    }
                }
            }

            #[allow(non_snake_case)]
            static #definition_getter_instance: #definition_getter_struct = #definition_getter_struct;

            #[allow(non_snake_case)]
            #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
            #[::rorm::rename_linkme]
            static #definition_getter_dyn_obj: &'static dyn ::rorm::model::GetModelDefinition = &#definition_getter_instance;

            #errors
        }
    })
}

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

/// Parse the `#[rorm(max_length = ..)]` annotation.
///
/// It accepts a single integer literal as argument.
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

/// Parse the `#[rorm(choices(..))]` annotation.
///
/// It accepts any number of string literals as arguments.
fn parse_choices(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    let usage_string =
        "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]";

    // Check if used as list i.e. "function call"
    if let syn::Meta::List(syn::MetaList { nested, .. }) = meta {
        let mut choices = Vec::new();

        // Check and collect string literals
        for choice in nested.iter() {
            match choice {
                syn::NestedMeta::Lit(syn::Lit::Str(choice)) => {
                    choices.push(choice);
                }
                _ => {
                    errors.push_new(choice.span(), usage_string);
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
        errors.push_new(meta.span(), usage_string);
    }
}

/// Parse the `#[rorm(index)]` annotation.
///
/// It accepts four different syntax's:
/// - `#[rorm(index)]`
/// - `#[rorm(index())]`
///    *(semantically identical to first one)*
/// - `#[rorm(name = <string literal>)]`
/// - `#[rorm(name = <string literal>, priority = <integer literal>)]`
///    *(insensitive to argument order)*
fn parse_index(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    match &meta {
        // index was used on its own without arguments
        syn::Meta::Path(_) => {
            annotations.push(quote! {
                ::rorm::imr::Annotation::Index(None)
            });
        }

        // index was used as "function call"
        syn::Meta::List(syn::MetaList { nested, .. }) => {
            let mut name = None;
            let mut prio = None;

            // Loop over arguments extracting `name` and `prio` while reporting any errors
            for nested_meta in nested.into_iter() {
                // Only accept keyword arguments
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

                // Only accept keywords who are identifier
                let ident = if let Some(ident) = path.get_ident() {
                    ident
                } else {
                    errors.push_new(
                        nested_meta.span(),
                        "index expects keyword arguments: #[rorm(index(name = \"...\"))]",
                    );
                    continue;
                };

                // Only accept "name" and "prio" as keywords
                // Check the associated value's type
                // Report duplications
                if ident == "name" {
                    if name.is_none() {
                        match literal {
                            syn::Lit::Str(literal) => {
                                name = Some(literal);
                            }
                            _ => {
                                errors.push_new(
                                    literal.span(),
                                    "name expects a string literal: #[rorm(index(name = \"...\"))]",
                                );
                            }
                        }
                    } else {
                        errors.push_new(ident.span(), "name has already been set");
                    }
                } else if ident == "priority" {
                    if prio.is_none() {
                        match literal {
                            syn::Lit::Int(literal) => {
                                prio = Some(literal);
                            }
                            _ => {
                                errors.push_new(literal.span(), "priority expects a integer literal: #[rorm(index(priority = \"...\"))]");
                            }
                        };
                    } else {
                        errors.push_new(ident.span(), "priority has already been set");
                    }
                } else {
                    errors.push_new(ident.span(), "unknown keyword argument");
                }
            }

            // Produce output depending on the 4 possible configurations
            // of `prio.is_some()` and `name.is_some()`
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

        // index was used as keyword argument
        _ => {
            errors.push_new(meta.span(), "index ether stands on its own or looks like a function call: #[rorm(index)] or #[rorm(index(..))]");
        }
    }
}
