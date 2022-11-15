use darling::{FromAttributes, FromMeta};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Lit, LitInt, LitStr, NestedMeta};

use crate::errors::Errors;

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct Annotations {
    pub auto_create_time: bool,
    pub auto_update_time: bool,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub id: bool,
    pub on_delete: Option<LitStr>,
    pub on_update: Option<LitStr>,

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
    pub default: Option<Lit>,

    /// Parse the `#[rorm(max_length = ..)]` annotation.
    ///
    /// It accepts a single integer literal as argument.
    pub max_length: Option<LitInt>,

    /// Parse the `#[rorm(choices(..))]` annotation.
    ///
    /// It accepts any number of string literals as arguments.
    pub choices: Option<Choices>,

    /// Parse the `#[rorm(index)]` annotation.
    ///
    /// It accepts four different syntax's:
    /// - `#[rorm(index)]`
    /// - `#[rorm(index())]`
    ///    *(semantically identical to first one)*
    /// - `#[rorm(index(name = <string literal>))]`
    /// - `#[rorm(index(name = <string literal>, priority = <integer literal>))]`
    ///    *(insensitive to argument order)*
    pub index: Option<Index>,
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

impl Annotations {
    pub fn to_tokens(mut self, errors: &Errors) -> TokenStream {
        if self.id {
            self.auto_increment = true;
            self.primary_key = true;
        }

        // Ensure every field is handled
        let Annotations {
            auto_create_time,
            auto_update_time,
            auto_increment,
            primary_key,
            unique,
            id: _, // Handled above
            on_delete,
            on_update,
            default,
            max_length,
            choices,
            index,
        } = self;

        // Convert every field into its "creation" expression
        let auto_create_time = auto_create_time.then(|| quote! {AutoCreateTime});
        let auto_update_time = auto_update_time.then(|| quote! {AutoUpdateTime});
        let auto_increment = auto_increment.then(|| quote! {AutoIncrement});
        let primary_key = primary_key.then(|| quote! {PrimaryKey});
        let unique = unique.then(|| quote! {Unique});
        let max_length = max_length.map(|len| quote! {MaxLength(#len)});
        let choices = choices.map(|Choices(choices)| quote! { Choices(&[#(#choices),*]) });
        let default = default
            .map(|default| {
                let variant = match &default {
                    Lit::Str(_) => "String",
                    Lit::Int(_) => "Integer",
                    Lit::Float(_) => "Float",
                    Lit::Bool(_) => "Boolean",
                    _ => {
                        errors.push_new(default.span(), "unsupported literal");
                        return None;
                    }
                };

                let variant = Ident::new(variant, Span::call_site());
                Some(quote! {DefaultValue(::rorm::hmr::annotations::DefaultValueData::#variant(#default))})
            })
            .flatten();
        let index = index.map(|Index(index)| {
            match index {
                None => {
                    quote! {Index(None)}
                }

                Some(NamedIndex {
                    name,
                    priority: None,
                }) => {
                    quote! { Index(Some(::rorm::hmr::annotations::IndexData { name: #name, priority: None })) }
                }

                Some(NamedIndex {
                    name,
                    priority: Some(priority),
                }) => {
                    quote! { Index(Some(::rorm::hmr::annotations::IndexData { name: #name, priority: Some(#priority) })) }
                }
            }
        });
        let parse_action = |lit: LitStr| match lit.value().as_str() {
            "Restrict" => Some(quote! {Restrict}),
            "Cascade" => Some(quote! {Cascade}),
            "SetNull" => Some(quote! {SetNull}),
            "SetDefault" => Some(quote! {SetDefault}),
            _ => {
                errors.push_new(lit.span(), "unsupported literal's value");
                return None;
            }
        };
        let on_delete = on_delete
            .map(parse_action)
            .flatten()
            .map(|token| quote! {OnDelete::#token});
        let on_update = on_update
            .map(parse_action)
            .flatten()
            .map(|token| quote! {OnUpdate::#token});

        // Unwrap all options
        // Add absolute path
        let finalize = |token: Option<TokenStream>| {
            if let Some(token) = token {
                quote! {Some(::rorm::hmr::annotations::#token)}
            } else {
                quote! {None}
            }
        };
        let auto_create_time = finalize(auto_create_time);
        let auto_update_time = finalize(auto_update_time);
        let auto_increment = finalize(auto_increment);
        let choices = finalize(choices);
        let default = finalize(default);
        let index = finalize(index);
        let max_length = finalize(max_length);
        let on_delete = finalize(on_delete);
        let on_update = finalize(on_update);
        let primary_key = finalize(primary_key);
        let unique = finalize(unique);

        // Combine into final struct
        quote! {
            ::rorm::annotations::Annotations {
                auto_create_time: #auto_create_time,
                auto_update_time: #auto_update_time,
                auto_increment: #auto_increment,
                choices: #choices,
                default: #default,
                index: #index,
                max_length: #max_length,
                on_delete: #on_delete,
                on_update: #on_update,
                primary_key: #primary_key,
                unique: #unique,
            }
        }
    }
}
