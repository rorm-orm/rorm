use std::ops::{Deref, DerefMut};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::Meta;

use crate::errors::Errors;

pub struct ParsedAnnotation {
    pub span: Span,
    pub annotation: Annotation,
    pub expr: Option<TokenStream>,
}

pub struct Annotations(Vec<ParsedAnnotation>);
impl Annotations {
    pub const fn new() -> Self {
        Annotations(Vec::new())
    }

    pub fn iter_steps<'a>(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.iter().map(
            |ParsedAnnotation {
                 annotation,
                 span,
                 expr,
             }| {
                let field = Ident::new(annotation.field(), span.clone());
                let variant = Ident::new(annotation.variant(), span.clone());
                if let Some(expr) = expr.as_ref() {
                    quote! {
                        .#field(::rorm::hmr::annotations::#variant(#expr))
                    }
                } else {
                    quote! {
                        .#field(::rorm::hmr::annotations::#variant)
                    }
                }
            },
        )
    }

    pub fn get_type(&self, value_type: &syn::Type) -> TokenStream {
        let mut anno_type = quote! {
            <#value_type as ::rorm::model::AsDbType>::Annotations
        };
        for ParsedAnnotation {
            annotation, span, ..
        } in self.iter()
        {
            let anno = Ident::new(annotation.variant(), span.clone());
            anno_type = quote! {
                ::rorm::annotation_builder::Add<::rorm::hmr::annotations::#anno, #anno_type>
            };
        }
        anno_type
    }
}
impl Deref for Annotations {
    type Target = Vec<ParsedAnnotation>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Annotations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Annotation {
    AutoCreateTime,
    AutoUpdateTime,
    AutoIncrement,
    Choices,
    Default,
    Index,
    MaxLength,
    PrimaryKey,
    Unique,
}
impl Annotation {
    pub fn variant(&self) -> &'static str {
        match self {
            Annotation::AutoCreateTime => "AutoCreateTime",
            Annotation::AutoUpdateTime => "AutoUpdateTime",
            Annotation::AutoIncrement => "AutoIncrement",
            Annotation::Choices => "Choices",
            Annotation::Default => "DefaultValue",
            Annotation::Index => "Index",
            Annotation::MaxLength => "MaxLength",
            Annotation::PrimaryKey => "PrimaryKey",
            Annotation::Unique => "Unique",
        }
    }

    pub fn field(&self) -> &'static str {
        match self {
            Annotation::AutoCreateTime => "auto_create_time",
            Annotation::AutoUpdateTime => "auto_update_time",
            Annotation::AutoIncrement => "auto_increment",
            Annotation::Choices => "choices",
            Annotation::Default => "default",
            Annotation::Index => "index",
            Annotation::MaxLength => "max_length",
            Annotation::PrimaryKey => "primary_key",
            Annotation::Unique => "unique",
        }
    }
}

pub trait ParseAnnotation: Sync {
    fn applies(&self, ident: &Ident) -> bool;
    fn parse(&self, ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors);
}

impl ParseAnnotation for Annotation {
    fn applies(&self, ident: &Ident) -> bool {
        ident == self.field()
    }

    fn parse(&self, ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        if let Meta::Path(_) = meta {
            annotations.push(ParsedAnnotation {
                span: ident.span(),
                annotation: *self,
                expr: None,
            });
        } else {
            errors.push_new(
                ident.span(),
                format!(
                    "{} doesn't take any values: #[rorm({})]",
                    self.field(),
                    self.field()
                ),
            );
        }
    }
}

struct ParseId;
impl ParseAnnotation for ParseId {
    fn applies(&self, ident: &Ident) -> bool {
        ident == "id"
    }

    /// Parse the `#[rorm(id)]` annotation
    fn parse(&self, ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        if let Meta::Path(_) = meta {
            annotations.push(ParsedAnnotation {
                span: ident.span(),
                annotation: Annotation::AutoIncrement,
                expr: None,
            });
            annotations.push(ParsedAnnotation {
                span: ident.span(),
                annotation: Annotation::PrimaryKey,
                expr: None,
            });
        } else {
            errors.push_new(meta.span(), "id doesn't take any values: #[rorm(id)]");
        }
    }
}

struct ParseDefault;
impl ParseAnnotation for ParseDefault {
    fn applies(&self, ident: &Ident) -> bool {
        ident == Annotation::Default.field()
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
    fn parse(&self, _ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        let arg = match meta {
            Meta::NameValue(syn::MetaNameValue { lit, .. }) => lit,
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
        annotations.push(ParsedAnnotation {
            span: arg.span(),
            annotation: Annotation::Default,
            expr: Some(quote! {::rorm::hmr::annotations::DefaultValueData::#variant(#arg)}),
        });
    }
}

struct ParseMaxLength;
impl ParseAnnotation for ParseMaxLength {
    fn applies(&self, ident: &Ident) -> bool {
        ident == Annotation::MaxLength.field()
    }

    /// Parse the `#[rorm(max_length = ..)]` annotation.
    ///
    /// It accepts a single integer literal as argument.
    fn parse(&self, _ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        match meta {
            Meta::NameValue(syn::MetaNameValue {
                lit: syn::Lit::Int(integer),
                ..
            }) => {
                annotations.push(ParsedAnnotation {
                    span: meta.span(),
                    annotation: Annotation::MaxLength,
                    expr: Some(quote! {
                        #integer
                    }),
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
}

struct ParseChoices;
impl ParseAnnotation for ParseChoices {
    fn applies(&self, ident: &Ident) -> bool {
        ident == Annotation::Choices.field()
    }

    /// Parse the `#[rorm(choices(..))]` annotation.
    ///
    /// It accepts any number of string literals as arguments.
    fn parse(&self, _ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        let usage_string =
            "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]";

        // Check if used as list i.e. "function call"
        if let Meta::List(syn::MetaList { nested, .. }) = meta {
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

            annotations.push(ParsedAnnotation {
                span: meta.span(),
                annotation: Annotation::Choices,
                expr: Some(quote! {
                    (&[#(#choices),*])
                }),
            });
        } else {
            errors.push_new(meta.span(), usage_string);
        }
    }
}

struct ParseIndex;
impl ParseAnnotation for ParseIndex {
    fn applies(&self, ident: &Ident) -> bool {
        ident == Annotation::Index.field()
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
    fn parse(&self, _ident: &Ident, meta: &Meta, annotations: &mut Annotations, errors: &Errors) {
        match &meta {
            // index was used on its own without arguments
            Meta::Path(_) => {
                annotations.push(ParsedAnnotation {
                    span: meta.span(),
                    annotation: Annotation::Index,
                    expr: Some(quote! {(None)}),
                });
            }

            // index was used as "function call"
            Meta::List(syn::MetaList { nested, .. }) => {
                let mut name = None;
                let mut prio = None;

                // Loop over arguments extracting `name` and `prio` while reporting any errors
                for nested_meta in nested.into_iter() {
                    // Only accept keyword arguments
                    let (path, literal) =
                        if let syn::NestedMeta::Meta(Meta::NameValue(syn::MetaNameValue {
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
                        quote! { Some(::rorm::hmr::annotations::IndexData { name: #name, priority: #prio }) }
                    } else {
                        quote! { None }
                    };
                    annotations.push(ParsedAnnotation {
                        span: meta.span(),
                        annotation: Annotation::Index,
                        expr: Some(quote! {(#inner)}),
                    });
                }
            }

            // index was used as keyword argument
            _ => {
                errors.push_new(meta.span(), "index ether stands on its own or looks like a function call: #[rorm(index)] or #[rorm(index(..))]");
            }
        }
    }
}

struct ParseUnknown;
impl ParseAnnotation for ParseUnknown {
    /// This remaining catch all annotation always applies
    fn applies(&self, _ident: &Ident) -> bool {
        true
    }

    fn parse(&self, ident: &Ident, _meta: &Meta, _annotations: &mut Annotations, errors: &Errors) {
        errors.push_new(ident.span(), "Unknown annotation")
    }
}

// TODO redesign trait objects with simple function pointers?
pub static ANNOTATIONS: &[&dyn ParseAnnotation] = &[
    &Annotation::AutoCreateTime,
    &Annotation::AutoUpdateTime,
    &Annotation::PrimaryKey,
    &Annotation::Unique,
    &Annotation::AutoIncrement,
    &ParseId,
    &ParseDefault,
    &ParseMaxLength,
    &ParseChoices,
    &ParseIndex,
    &ParseUnknown,
];
