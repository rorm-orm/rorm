use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Type, Visibility};

pub fn patch(
    vis: &Visibility,
    strct: &Ident,
    model: &impl ToTokens,
    fields: &[Ident],
    types: &[Type],
) -> TokenStream {
    let patch_decoder = format_ident!("__{strct}_Decoder");
    quote! {
        const _: () = {
            use ::rorm::internal::field::{AbstractField, FieldType};
            use ::rorm::internal::field::decoder::FieldDecoder;

            #vis struct #patch_decoder {
                #(
                    #fields: <#types as ::rorm::internal::field::FieldType>::Decoder,
                )*
            }
            impl ::rorm::crud::decoder::Decoder for #patch_decoder
            {
                type Result = #strct;

                fn by_name(&self, row: &::rorm::row::Row) -> Result<Self::Result, ::rorm::Error> {
                    Ok(#strct {#(
                        #fields: self.#fields.by_name(row)?,
                    )*})
                }

                fn by_index(&self, row: &::rorm::row::Row) -> Result<Self::Result, ::rorm::Error> {
                    Ok(#strct {#(
                        #fields: self.#fields.by_index(row)?,
                    )*})
                }
            }

            impl ::rorm::model::Patch for #strct {
                type Model = #model;

                type Decoder = #patch_decoder;

                fn select<P: ::rorm::internal::relation_path::Path>(ctx: &mut ::rorm::internal::query_context::QueryContext) -> Self::Decoder {
                    #patch_decoder {#(
                        #fields: FieldDecoder::new(
                            ctx,
                            <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.through::<P>(),
                        ),
                    )*}
                }

                const COLUMNS: &'static [&'static str] = ::rorm::concat_columns!(&[#(
                    ::rorm::internal::field::FieldProxy::columns(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields),
                )*]);

                fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
                    use ::rorm::internal::field::AbstractField;
                    #(
                        values.extend(self.#fields.as_values());
                    )*
                }

                fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
                    #(
                        values.extend(self.#fields.into_values());
                    )*
                }
            }

            impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for #strct {
                type Patch = #strct;

                fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, #strct> {
                    ::rorm::internal::patch::PatchCow::Owned(self)
                }
            }
            impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a #strct {
                type Patch = #strct;

                fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, #strct> {
                    ::rorm::internal::patch::PatchCow::Borrowed(self)
                }
            }

            #(
                impl ::rorm::model::GetField<::rorm::get_field!(#strct, #fields)> for #strct {
                    fn get_field(self) -> <::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::Type {
                        self.#fields
                    }
                    fn borrow_field(&self) -> &<::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::Type {
                        &self.#fields
                    }
                    fn borrow_field_mut(&mut self) -> &mut <::rorm::get_field!(#strct, #fields) as ::rorm::internal::field::RawField>::Type {
                        &mut self.#fields
                    }
                }
            )*
        };
    }
}
