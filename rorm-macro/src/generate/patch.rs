use std::array;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Type, Visibility};

use crate::parse::patch::ParsedPatch;

pub fn generate_patch(patch: &ParsedPatch) -> TokenStream {
    let ParsedPatch {
        vis,
        ident,
        model,
        fields,
    } = patch;

    let field_idents_1 = fields.iter().map(|field| &field.ident);
    let field_idents_2 = field_idents_1.clone();
    let field_types = fields.iter().map(|field| &field.ty);

    let partial = partially_generate_patch(
        ident,
        model,
        vis,
        field_idents_1.clone(),
        fields.iter().map(|field| &field.ty),
    );

    quote! {
        const _: () = {
            #partial

            #(
                impl ::rorm::model::GetField<::rorm::get_field!(#ident, #field_idents_2)> for #ident {
                    fn get_field(self) -> #field_types {
                        self.#field_idents_2
                    }
                    fn borrow_field(&self) -> &#field_types {
                        &self.#field_idents_2
                    }
                    fn borrow_field_mut(&mut self) -> &mut #field_types {
                        &mut self.#field_idents_2
                    }
                }
            )*
        };
    }
}

pub fn partially_generate_patch<'a>(
    patch: &Ident,
    model: &impl ToTokens, // Ident or Path
    vis: &Visibility,
    fields: impl Iterator<Item = &'a Ident> + Clone,
    types: impl Iterator<Item = &'a Type> + Clone,
) -> TokenStream {
    let decoder = format_ident!("__{patch}_Decoder");
    let [fields_1, fields_2, fields_3, fields_4, fields_5, fields_6, fields_7] =
        array::from_fn(|_| fields.clone());
    quote! {
        use ::rorm::internal::field::AbstractField;
        use ::rorm::internal::field::decoder::FieldDecoder;
        use ::rorm::fields::traits::FieldType;

        #vis struct #decoder {
            #(
                #fields_1: <#types as ::rorm::fields::traits::FieldType>::Decoder,
            )*
        }
        impl ::rorm::crud::decoder::Decoder for #decoder {
            type Result = #patch;

            fn by_name(&self, row: &::rorm::row::Row) -> Result<Self::Result, ::rorm::Error> {
                Ok(#patch {#(
                    #fields_2: self.#fields_2.by_name(row)?,
                )*})
            }

            fn by_index(&self, row: &::rorm::row::Row) -> Result<Self::Result, ::rorm::Error> {
                Ok(#patch {#(
                    #fields_3: self.#fields_3.by_index(row)?,
                )*})
            }
        }

        impl ::rorm::model::Patch for #patch {
            type Model = #model;

            type Decoder = #decoder;

            fn select<P: ::rorm::internal::relation_path::Path>(ctx: &mut ::rorm::internal::query_context::QueryContext) -> Self::Decoder {
                #decoder {#(
                    #fields_4: FieldDecoder::new(
                        ctx,
                        <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields_4.through::<P>(),
                    ),
                )*}
            }

            const COLUMNS: &'static [&'static str] = ::rorm::concat_columns!(&[#(
                ::rorm::internal::field::FieldProxy::columns(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields_5),
            )*]);

            fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
                #(
                    values.extend(self.#fields_6.as_values());
                )*
            }

            fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
                #(
                    values.extend(self.#fields_7.into_values());
                )*
            }
        }

        impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for #patch {
            type Patch = #patch;

            fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, #patch> {
                ::rorm::internal::patch::PatchCow::Owned(self)
            }
        }
        impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a #patch {
            type Patch = #patch;

            fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, #patch> {
                ::rorm::internal::patch::PatchCow::Borrowed(self)
            }
        }
    }
}
