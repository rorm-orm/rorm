use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

pub fn patch<'a>(strct: &Ident, model: &impl ToTokens, field_idents: &[Ident]) -> TokenStream {
    quote! {
        impl ::rorm::model::Patch for #strct {
            type Model = #model;

            const COLUMNS: &'static [&'static str] = &[#(
                stringify!(#field_idents),
            )*];

            const INDEXES: &'static [usize] = &[#(
                <Self as ::rorm::model::Patch>::Model::FIELDS.#field_idents.index,
            )*];
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

pub fn into_column_iterator(patch: &Ident, columns: &[Ident]) -> TokenStream {
    let iter_ident = format_ident!("__{}Iterator", patch);
    let index = columns
        .iter()
        .enumerate()
        .map(|(index, column)| syn::LitInt::new(&index.to_string(), column.span()));
    quote! {
        impl<'a> ::rorm::model::IntoColumnIterator<'a> for &'a #patch {
            type Iterator = #iter_ident<'a>;

            fn into_column_iter(self) -> Self::Iterator {
                #iter_ident {
                    next: 0,
                    patch: self,
                }
            }
        }
        pub struct #iter_ident<'a> {
            next: usize,
            patch: &'a #patch,
        }
        impl<'a> Iterator for #iter_ident<'a> {
            type Item = ::rorm::value::Value<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                use ::rorm::model::AsDbType;
                self.next += 1;
                match self.next - 1 {
                    #(
                        #index => Some(self.patch.#columns.as_primitive()),
                    )*
                    _ => None,
                }
            }
        }
    }
}
