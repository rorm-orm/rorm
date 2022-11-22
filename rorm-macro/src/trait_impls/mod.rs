use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub fn patch(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [&'static str] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.name(),
            )*];

            const INDEXES: &'static [usize] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index(),
            )*];

            fn get(&self, index: usize) -> Option<::rorm::conditions::Value> {
                use ::rorm::internal::as_db_type::AsDbType;
                #(
                    if index == <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.index() {
                        <Self as ::rorm::model::Patch>::Model::FIELDS.#fields.get_value(&self.#fields)
                    } else
                )* {
                    None
                }
            }
        }
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
                        #fields: <#model as ::rorm::model::Model>::FIELDS.#fields.get_from_row(
                            &row,
                            <#model as ::rorm::model::Model>::FIELDS.#fields.name()
                        )?,
                    )*
                    #(
                        #ignored: Default::default(),
                    )*
                })
            }
        }
    }
}
