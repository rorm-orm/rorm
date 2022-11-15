use darling::{FromAttributes, FromMeta};
use std::ops::{Deref, DerefMut};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Lit, LitInt, LitStr, NestedMeta};

use crate::errors::Errors;

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct FieldAnnotations {
    auto_create_time: bool,
    auto_update_time: bool,
    auto_increment: bool,
    primary_key: bool,
    unique: bool,
    id: bool,

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
    default: Option<Lit>,

    /// Parse the `#[rorm(max_length = ..)]` annotation.
    ///
    /// It accepts a single integer literal as argument.
    max_length: Option<LitInt>,

    /// Parse the `#[rorm(choices(..))]` annotation.
    ///
    /// It accepts any number of string literals as arguments.
    choices: Option<Choices>,

    /// Parse the `#[rorm(index)]` annotation.
    ///
    /// It accepts four different syntax's:
    /// - `#[rorm(index)]`
    /// - `#[rorm(index())]`
    ///    *(semantically identical to first one)*
    /// - `#[rorm(index(name = <string literal>))]`
    /// - `#[rorm(index(name = <string literal>, priority = <integer literal>))]`
    ///    *(insensitive to argument order)*
    index: Option<Index>,
}

#[derive(Debug)]
pub struct Choices(Vec<LitStr>);
impl FromMeta for Choices {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let result: darling::Result<Vec<LitStr>> = items
            .iter()
            .map(<LitStr as FromMeta>::from_nested_meta)
            .collect();
        result.map(Choices)
    }
}

#[derive(Default, Debug)]
pub struct Index(Option<NamedIndex>);
impl FromMeta for Index {
    fn from_word() -> darling::Result<Self> {
        Ok(Index(None))
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            Ok(Index(None))
        } else {
            Ok(Index(Some(NamedIndex::from_list(items)?)))
        }
    }
}

#[derive(FromMeta, Debug)]
pub struct NamedIndex {
    name: LitStr,
    priority: Option<LitInt>,
}

impl FieldAnnotations {
    pub fn into_with_error(self, field: &Span, errors: &Errors) -> Annotations {
        let FieldAnnotations {
            auto_create_time,
            auto_update_time,
            auto_increment,
            primary_key,
            unique,
            id,
            default,
            max_length,
            choices,
            index,
        } = self;

        let mut annotations: Vec<(Annotation, Option<TokenStream>)> = Vec::new();
        if auto_create_time {
            annotations.push((Annotation::AutoCreateTime, None));
        }
        if auto_update_time {
            annotations.push((Annotation::AutoUpdateTime, None));
        }
        if auto_increment {
            annotations.push((Annotation::AutoIncrement, None));
        }
        if primary_key {
            annotations.push((Annotation::PrimaryKey, None));
        }
        if unique {
            annotations.push((Annotation::Unique, None));
        }
        if id {
            annotations.push((Annotation::AutoIncrement, None));
            annotations.push((Annotation::PrimaryKey, None));
        }
        if let Some(default) = default {
            let variant = match &default {
                Lit::Str(_) => "String",
                Lit::Int(_) => "Integer",
                Lit::Float(_) => "Float",
                Lit::Bool(_) => "Boolean",
                _ => {
                    errors.push_new(default.span(), "unsupported literal");
                    ""
                }
            };

            if !variant.is_empty() {
                let variant = Ident::new(variant, *field);
                annotations.push((
                    Annotation::Default,
                    Some(quote! {::rorm::hmr::annotations::DefaultValueData::#variant(#default)}),
                ));
            }
        }
        if let Some(max_length) = max_length {
            annotations.push((Annotation::MaxLength, Some(max_length.to_token_stream())));
        }
        if let Some(Index(index)) = index {
            let expr = match index {
                None => {
                    quote! {(None)}
                }

                Some(NamedIndex {
                    name,
                    priority: None,
                }) => {
                    quote! { Some(::rorm::hmr::annotations::IndexData { name: #name, priority: None }) }
                }

                Some(NamedIndex {
                    name,
                    priority: Some(priority),
                }) => {
                    quote! { Some(::rorm::hmr::annotations::IndexData { name: #name, priority: Some(#priority) }) }
                }
            };
            annotations.push((Annotation::Index, Some(expr)));
        }
        if let Some(Choices(choices)) = choices {
            annotations.push((
                Annotation::Choices,
                Some(quote! {
                    (&[#(#choices),*])
                }),
            ));
        }

        Annotations(
            annotations
                .into_iter()
                .map(|(annotation, expr)| ParsedAnnotation {
                    span: *field,
                    annotation,
                    expr,
                })
                .collect(),
        )
    }
}

pub struct ParsedAnnotation {
    pub span: Span,
    pub annotation: Annotation,
    pub expr: Option<TokenStream>,
}

pub struct Annotations(Vec<ParsedAnnotation>);
impl Annotations {
    pub fn iter_steps(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.iter().map(
            |ParsedAnnotation {
                 annotation,
                 span,
                 expr,
             }| {
                let field = Ident::new(annotation.field(), *span);
                let variant = Ident::new(annotation.variant(), *span);
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
            <#value_type as ::rorm::internal::as_db_type::AsDbType>::Annotations
        };
        for ParsedAnnotation {
            annotation, span, ..
        } in self.iter()
        {
            let anno = Ident::new(annotation.variant(), *span);
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
