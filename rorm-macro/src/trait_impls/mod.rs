use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub fn patch(strct: &Ident, model: &impl ToTokens, fields: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [Option<&'static str>] = &[#(
                <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.name(),
            )*];

            const INDEXES: &'static [usize] = &[#(
                <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.index(),
            )*];

            fn push_values<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
                #(
                    <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.push_value(&self.#fields, values);
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
                        #fields: <#model as ::rorm::model::Model>::FIELDS.#fields.get_from_row(&row, Option::<usize>::None)?,
                    )*
                    #(
                        #ignored: Default::default(),
                    )*
                })
            }
            fn from_row_using_position(row: ::rorm::row::Row) -> Result<Self, ::rorm::Error> {
                let mut i: isize = -1;
                Ok(#strct {
                    #(
                        #fields: {
                            if <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.name().is_some() {
                                i += 1;
                                <#model as ::rorm::model::Model>::FIELDS.#fields.get_from_row(&row, Some(i as usize))?
                            } else {
                                <#model as ::rorm::model::Model>::FIELDS.#fields.get_from_row(&row, Option::<usize>::None)?
                            }
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
