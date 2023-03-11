use darling::{Error, FromAttributes, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_token, ToTokens};
use syn::{Lit, LitInt, LitStr, NestedMeta};

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct Annotations {
    pub auto_create_time: bool,
    pub auto_update_time: bool,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub id: bool,
    pub on_delete: Option<OnAction>,
    pub on_update: Option<OnAction>,
    pub rename: Option<LitStr>,
    pub ignore: bool,

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
    pub default: Option<Default>,

    /// Parse the `#[rorm(max_length = ..)]` annotation.
    ///
    /// It accepts a single integer literal as argument.
    pub max_length: Option<LitInt>,

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
pub struct Default {
    variant: &'static str,
    literal: Lit,
}
impl FromMeta for Default {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(Default {
            variant: match value {
                Lit::Str(_) => Ok("String"),
                Lit::Int(_) => Ok("Integer"),
                Lit::Float(_) => Ok("Float"),
                Lit::Bool(_) => Ok("Boolean"),
                _ => Err(Error::unexpected_lit_type(value)),
            }?,
            literal: value.clone(),
        })
    }
}

#[derive(Debug)]
pub struct OnAction(Ident);
impl FromMeta for OnAction {
    fn from_value(lit: &Lit) -> darling::Result<Self> {
        static OPTIONS: [&str; 4] = ["Restrict", "Cascade", "SetNull", "SetDefault"];
        (match lit {
            Lit::Str(string) => {
                let string = string.value();
                let value = string.as_str();
                if OPTIONS.contains(&value) {
                    Ok(OnAction(Ident::new(value, lit.span())))
                } else {
                    Err(Error::unknown_field_with_alts(value, &OPTIONS))
                }
            }
            _ => Err(Error::unexpected_lit_type(lit)),
        })
        .map_err(|e| e.with_span(lit))
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

impl ToTokens for Annotations {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        // Ensure every field is handled
        let Annotations {
            auto_create_time,
            auto_update_time,
            mut auto_increment,
            mut primary_key,
            unique,
            id, // Handled above
            on_delete,
            on_update,
            rename: _, // Not db annotations
            ignore: _, //
            default,
            max_length,
            index,
        } = self;

        if *id {
            auto_increment = true;
            primary_key = true;
        }

        // Convert every field into its "creation" expression
        let auto_create_time = auto_create_time.then(|| quote! {AutoCreateTime});
        let auto_update_time = auto_update_time.then(|| quote! {AutoUpdateTime});
        let auto_increment = auto_increment.then(|| quote! {AutoIncrement});
        let primary_key = primary_key.then(|| quote! {PrimaryKey});
        let unique = unique.then(|| quote! {Unique});
        let max_length = max_length.as_ref().map(|len| quote! {MaxLength(#len)});
        let default = default.as_ref().map(|Default { variant, literal }| {
            let variant = Ident::new(variant, literal.span());
            quote! {DefaultValue(::rorm::internal::hmr::annotations::DefaultValueData::#variant(#literal))}
        });
        let index = index.as_ref().map(|Index(index)| {
            match index {
                None => {
                    quote! {Index(None)}
                }

                Some(NamedIndex {
                    name,
                    priority: None,
                }) => {
                    quote! { Index(Some(::rorm::internal::hmr::annotations::IndexData { name: #name, priority: None })) }
                }

                Some(NamedIndex {
                    name,
                    priority: Some(priority),
                }) => {
                    quote! { Index(Some(::rorm::internal::hmr::annotations::IndexData { name: #name, priority: Some(#priority) })) }
                }
            }
        });
        let on_delete = on_delete
            .as_ref()
            .map(|OnAction(token)| quote! {OnDelete::#token});
        let on_update = on_update
            .as_ref()
            .map(|OnAction(token)| quote! {OnUpdate::#token});

        // Unwrap all options
        // Add absolute path
        let finalize = |token: Option<TokenStream>| {
            if let Some(token) = token {
                quote! {Some(::rorm::internal::hmr::annotations::#token)}
            } else {
                quote! {None}
            }
        };
        let auto_create_time = finalize(auto_create_time);
        let auto_update_time = finalize(auto_update_time);
        let auto_increment = finalize(auto_increment);
        let default = finalize(default);
        let index = finalize(index);
        let max_length = finalize(max_length);
        let on_delete = finalize(on_delete);
        let on_update = finalize(on_update);
        let primary_key = finalize(primary_key);
        let unique = finalize(unique);

        // Combine into final struct
        quote_token! {{
            ::rorm::internal::hmr::annotations::Annotations {
                auto_create_time: #auto_create_time,
                auto_update_time: #auto_update_time,
                auto_increment: #auto_increment,
                choices: None, // Set implicitly by type
                default: #default,
                index: #index,
                max_length: #max_length,
                on_delete: #on_delete,
                on_update: #on_update,
                primary_key: #primary_key,
                unique: #unique,
                nullable: false, // Set implicitly by type
                foreign: None,   //
            }
        }tokens}
    }
}
