use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub fn patch(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [Option<&'static str>] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.name(),
            )*];

            const INDEXES: &'static [usize] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index(),
            )*];

            fn get_value(&self, index: usize) -> Option<::rorm::conditions::Value> {
                use ::rorm::internal::field::as_db_type::AsDbType;
                #(
                    if index == <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index() {
                        <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.get_value(&self.#fields)
                    } else
                )* {
                    None
                }
            }
        }

        #(
            impl ::rorm::model::GetField<::rorm::get_field!(#strct, #fields)> for #strct {
                fn get_field(&self) -> &<::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::RawType {
                    &self.#fields
                }
                fn get_field_mut(&mut self) -> &mut <::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::RawType {
                    &mut self.#fields
                }
            }
        )*
    }
}

pub fn try_from_row(
    strct: &Ident,
    model: &impl ToTokens,
    fields: &[Ident],
    ignored: &[Ident],
) -> TokenStream {
    quote! {
        impl ::rorm::row::FromRow for #strct {
            fn from_row(row: ::rorm::row::Row) -> Result<Self, ::rorm::Error> {
                Ok(#strct {
                    #(
                        #fields: <#model as ::rorm::model::Model>::FIELDS.#fields.get_from_row(&row, Option::<usize>::None)?,
                    )*
                    #(
                        #ignored: Default::default(),
                    )*
                })
            }
        }
    }
}
