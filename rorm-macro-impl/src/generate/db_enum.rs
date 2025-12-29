use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::db_enum::ParsedDbEnum;
use crate::MacroConfig;

pub fn generate_db_enum(parsed: &ParsedDbEnum, config: &MacroConfig) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

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

            impl #rorm_path::fields::traits::FieldType for #ident {
                type Columns = #rorm_path::fields::traits::Array<1>;

                const NULL: #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::db::sql::value::NullType> = [
                    #rorm_path::db::sql::value::NullType::Choice
                ];

                fn into_values<'a>(self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'a>> {
                    [#rorm_path::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                fn as_values(&self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'_>> {
                    [#rorm_path::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#variants => stringify!(#variants),
                        )*
                    }))]
                }

                type Decoder = #decoder;

                type GetAnnotations = get_db_enum_annotations;

                type Check = #rorm_path::fields::utils::check::shared_linter_check<1>;

                type GetNames = #rorm_path::fields::utils::get_names::single_column_name;
            }
            #rorm_path::new_converting_decoder!(
                #[doc(hidden)]
                #vis #decoder,
                |value: #rorm_path::db::choice::Choice| -> #ident {
                    let value: String = value.0;
                    match value.as_str() {
                        #(
                            stringify!(#variants) => Ok(#ident::#variants),
                        )*
                        _ => Err(format!("Invalid value '{}' for enum '{}'", value, stringify!(#ident))),
                    }
                }
            );
            impl<'a> #rorm_path::fields::traits::into_value::IntoValue<'a> for #ident {
                fn into_value(self) -> #rorm_path::conditions::Value<'a> {
                    let [value] = #rorm_path::fields::traits::FieldType::into_values(self);
                    value
                }
            }
            impl #rorm_path::fields::traits::simple::SimpleFieldEq<#ident> for #ident {}

            #rorm_path::const_fn! {
                pub fn get_db_enum_annotations(
                    field: #rorm_path::internal::hmr::annotations::Annotations
                ) -> [#rorm_path::internal::hmr::annotations::Annotations; 1] {
                    let mut field = field;
                    field.choices = Some(#rorm_path::internal::hmr::annotations::Choices(CHOICES));
                    [field]
                }
            }
        };
    }
}
