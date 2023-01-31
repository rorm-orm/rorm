use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub fn patch(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [&'static str] = ::rorm::concat_columns!(&[#(
                ::rorm::internal::field::FieldProxy::columns(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields),
            )*]);

            const INDEXES: &'static [usize] = &[#(
                ::rorm::internal::field::FieldProxy::index(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields),
            )*];

            fn push_values<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
                #(
                    ::rorm::internal::field::FieldProxy::push_value(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields, &self.#fields, values);
                )*
            }
        }

        #(
            impl ::rorm::model::GetField<::rorm::get_field!(#strct, #fields)> for #strct {
                fn get_field(&self) -> &<::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::Type {
                    &self.#fields
                }
                fn get_field_mut(&mut self) -> &mut <::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::Type {
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
                        #fields: ::rorm::internal::field::FieldProxy::get_by_name(<#model as ::rorm::model::Model>::FIELDS.#fields, &row)?,
                    )*
                    #(
                        #ignored: Default::default(),
                    )*
                })
            }
            fn from_row_using_position(row: ::rorm::row::Row) -> Result<Self, ::rorm::Error> {
                let mut i = 0;
                Ok(#strct {
                    #(
                        #fields: {
                            let value = ::rorm::internal::field::FieldProxy::get_by_index(<#model as ::rorm::model::Model>::FIELDS.#fields, &row, i as usize)?;
                            i += ::rorm::internal::field::FieldProxy::columns(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields).len();
                            value
                        },
                    )*
                    #(
                        #ignored: Default::default(),
                    )*
                })
            }
        }
    }
}
