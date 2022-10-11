use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub fn patch<'a>(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [&'static str] = &[#(
                stringify!(#fields),
            )*];

            const INDEXES: &'static [usize] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index,
            )*];

            fn get(&self, index: usize) -> Option<::rorm::value::Value> {
                use ::rorm::model::AsDbType;
                #(
                    if index == <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index {
                        Some(self.#fields.as_primitive())
                    } else
                )* {
                    None
                }
            }
        }
    }
}

pub fn try_from_row(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl TryFrom<::rorm::row::Row> for #strct {
            type Error = ::rorm::error::Error;

            fn try_from(row: ::rorm::row::Row) -> Result<Self, Self::Error> {
                Ok(#strct {
                    #(
                        #fields: <#model as ::rorm::model::Model>::FIELDS.#fields.convert_primitive(
                            row.get(<#model as ::rorm::model::Model>::FIELDS.#fields.name)?
                        ),
                    )*
                })
            }
        }
    }
}
