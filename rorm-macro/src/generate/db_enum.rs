use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::db_enum::ParsedDbEnum;

pub fn generate_db_enum(parsed: &ParsedDbEnum) -> TokenStream {
    let ParsedDbEnum { ident, variants } = parsed;
    let decoder = format_ident!("__{ident}_Decoder");

    quote! {
        const _: () = {
            const CHOICES: &'static [&'static str] = &[
                #(stringify!(#variants)),*
            ];

            impl ::rorm::internal::field::FieldType for #ident {
                type Kind = ::rorm::internal::field::kind::AsDbType;

                type Columns<'a> = [::rorm::conditions::Value<'a>; 1];

                fn into_values(self) -> Self::Columns<'static> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                fn as_values(&self) -> Self::Columns<'_> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                type Decoder = #decoder;
            }
            ::rorm::new_converting_decoder!(
                #[doc(hidden)]
                #decoder,
                |value: ::rorm::choice::Choice| -> #ident {
                    let value: String = value.0;
                    match value.as_str() {
                        #(
                            stringify!(#variants) => Ok(#ident::#variants),
                        )*
                        _ => Err(::rorm::Error::DecodeError(format!("Invalid value '{}' for enum '{}'", value, stringify!(#ident)))),
                    }
                }
            );
            impl ::rorm::internal::field::as_db_type::AsDbType for #ident {
                type Primitive = ::rorm::choice::Choice;
                type DbType = ::rorm::internal::hmr::db_type::Choices;

                const IMPLICIT: Option<::rorm::internal::hmr::annotations::Annotations> = Some({
                    let mut annos = ::rorm::internal::hmr::annotations::Annotations::empty();
                    annos.choices = Some(::rorm::internal::hmr::annotations::Choices(CHOICES));
                    annos
                });

                fn from_primitive(primitive: Self::Primitive) -> Self {
                    use #ident::*;
                    match primitive.0.as_str() {
                        #(stringify!(#variants) => #variants,)*
                        _ => panic!("Unexpected database value"),
                    }
                }
            }
            impl<'a> ::rorm::conditions::IntoSingleValue<'a, ::rorm::internal::hmr::db_type::Choices> for #ident {
                type Condition = ::rorm::conditions::Value<'a>;

                fn into_condition(self) -> Self::Condition {
                    ::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))
                }
            }
        };
    }
}
