use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::db_enum::ParsedDbEnum;

pub fn generate_db_enum(parsed: &ParsedDbEnum) -> TokenStream {
    let ParsedDbEnum {
        vis,
        ident,
        variants,
    } = parsed;
    let decoder = format_ident!("__{ident}_Decoder");

    quote! {
        const _: () = {
            const CHOICES: &'static [&'static str] = &[
                #(stringify!(#variants)),*
            ];

            impl ::rorm::fields::traits::FieldType for #ident {
                type Columns<T> = [T; 1];

                fn into_values(self) -> Self::Columns<::rorm::conditions::Value<'static>> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                fn as_values(&self) -> Self::Columns<::rorm::conditions::Value<'_>> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                type Decoder = #decoder;

                fn get_imr<F: ::rorm::internal::field::Field<Type = Self>>() -> Self::Columns<::rorm::internal::imr::Field> {
                    use ::rorm::internal::hmr::AsImr;
                    [::rorm::internal::imr::Field {
                        name: F::NAME.to_string(),
                        db_type: <::rorm::internal::hmr::db_type::Choices as ::rorm::internal::hmr::db_type::DbType>::IMR,
                        annotations: F::EFFECTIVE_ANNOTATIONS
                            .unwrap_or_else(::rorm::internal::hmr::annotations::Annotations::empty)
                            .as_imr(),
                        source_defined_at: F::SOURCE.map(|s| s.as_imr()),
                    }]
                }

                type AnnotationsModifier<F: ::rorm::internal::field::Field<Type = Self>> = ::rorm::internal::field::modifier::MergeAnnotations<Self>;

                type CheckModifier<F: ::rorm::internal::field::Field<Type = Self>> = ::rorm::internal::field::modifier::SingleColumnCheck<::rorm::internal::hmr::db_type::Choices>;

                type ColumnsFromName<F: ::rorm::internal::field::Field<Type = Self>> = ::rorm::internal::field::modifier::SingleColumnFromName;
            }
            ::rorm::new_converting_decoder!(
                #[doc(hidden)]
                #vis #decoder,
                |value: ::rorm::db::choice::Choice| -> #ident {
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
                type Primitive = ::rorm::db::choice::Choice;
                type DbType = ::rorm::internal::hmr::db_type::Choices;

                const IMPLICIT: Option<::rorm::internal::hmr::annotations::Annotations> = Some({
                    let mut annos = ::rorm::internal::hmr::annotations::Annotations::empty();
                    annos.choices = Some(::rorm::internal::hmr::annotations::Choices(CHOICES));
                    annos
                });
            }
            ::rorm::impl_FieldEq!(impl<'rhs> FieldEq<'rhs, #ident> for #ident {
                |value: #ident| { let [value] = <#ident as ::rorm::fields::traits::FieldType>::into_values(value); value }
            });
        };
    }
}
