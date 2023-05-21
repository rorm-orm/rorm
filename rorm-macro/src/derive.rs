use darling::{Error, FromAttributes, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::trait_impls;

pub fn db_enum(enm: TokenStream) -> darling::Result<TokenStream> {
    let enm = match syn::parse2::<syn::ItemEnum>(enm) {
        Ok(enm) => enm,
        Err(err) => return Ok(err.into_compile_error()),
    };

    let mut errors = Error::accumulator();
    let mut identifiers = Vec::new();
    for variant in enm.variants {
        if variant.fields.is_empty() {
            identifiers.push(variant.ident);
        } else {
            errors.push(
                Error::unsupported_shape("variants aren't allowed to contain data")
                    .with_span(&variant.fields),
            );
        }
    }
    let db_enum = enm.ident;
    let decoder = format_ident!("__{db_enum}_Decoder");

    errors.finish()?;
    Ok(quote! {
        const _: () = {
            const CHOICES: &'static [&'static str] = &[
                #(stringify!(#identifiers)),*
            ];

            impl ::rorm::internal::field::FieldType for #db_enum {
                type Kind = ::rorm::internal::field::kind::AsDbType;

                type Columns<'a> = [::rorm::conditions::Value<'a>; 1];

                fn into_values(self) -> Self::Columns<'static> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#identifiers => stringify!(#identifiers),
                        )*
                    }))]
                }

                fn as_values(&self) -> Self::Columns<'_> {
                    [::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#identifiers => stringify!(#identifiers),
                        )*
                    }))]
                }

                type Decoder = #decoder;
            }
            ::rorm::new_converting_decoder!(
                #[doc(hidden)]
                #decoder,
                |value: ::rorm::choice::Choice| -> #db_enum {
                    let value: String = value.0;
                    match value.as_str() {
                        #(
                            stringify!(#identifiers) => Ok(#db_enum::#identifiers),
                        )*
                        _ => Err(::rorm::Error::DecodeError(format!("Invalid value '{}' for enum '{}'", value, stringify!(#db_enum)))),
                    }
                }
            );
            impl ::rorm::internal::field::as_db_type::AsDbType for #db_enum {
                type Primitive = ::rorm::choice::Choice;
                type DbType = ::rorm::internal::hmr::db_type::Choices;

                const IMPLICIT: Option<::rorm::internal::hmr::annotations::Annotations> = Some({
                    let mut annos = ::rorm::internal::hmr::annotations::Annotations::empty();
                    annos.choices = Some(::rorm::internal::hmr::annotations::Choices(CHOICES));
                    annos
                });

                fn from_primitive(primitive: Self::Primitive) -> Self {
                    use #db_enum::*;
                    match primitive.0.as_str() {
                        #(stringify!(#identifiers) => #identifiers,)*
                        _ => panic!("Unexpected database value"),
                    }
                }
            }
            impl<'a> ::rorm::conditions::IntoSingleValue<'a, ::rorm::internal::hmr::db_type::Choices> for #db_enum {
                type Condition = ::rorm::conditions::Value<'a>;

                fn into_condition(self) -> Self::Condition {
                    ::rorm::conditions::Value::Choice(::std::borrow::Cow::Borrowed(match self {
                        #(
                            Self::#identifiers => stringify!(#identifiers),
                        )*
                    }))
                }
            }
        };
    })
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct PatchAnnotations {
    pub model: ModelPath,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct NoAnnotations;

#[derive(Debug)]
pub struct ModelPath(syn::Path);
impl FromMeta for ModelPath {
    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<syn::Path>(value)
            .map(ModelPath)
            .map_err(|error| Error::unknown_value(&error.to_string()))
    }
}

pub fn patch(strct: TokenStream) -> darling::Result<TokenStream> {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return Ok(err.into_compile_error()),
    };

    let mut errors = Error::accumulator();

    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();
    for field in strct.fields {
        errors.handle(NoAnnotations::from_attributes(&field.attrs));
        if let Some(ident) = field.ident {
            field_idents.push(ident);
            field_types.push(field.ty);
        } else {
            errors.push(Error::custom("missing field name").with_span(&field));
        }
    }

    let Some(PatchAnnotations {model: ModelPath(model_path)}) = errors.handle(PatchAnnotations::from_attributes(&strct.attrs)) else {
        return errors.finish_with(TokenStream::new());
    };

    let patch = strct.ident;
    let compile_check = format_ident!("__compile_check_{}", patch);
    let impl_patch =
        trait_impls::patch(&strct.vis, &patch, &model_path, &field_idents, &field_types);

    errors.finish()?;
    Ok(quote! {
        #[allow(non_snake_case)]
        fn #compile_check(model: #model_path) {
            // check fields exist on model and match model's types
            // todo error messages for type mismatches are terrible
            let _ = #patch {
                #(
                    #field_idents: model.#field_idents,
                )*
            };
        }

        #impl_patch
    })
}
