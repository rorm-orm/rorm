use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Type;

pub fn patch(
    strct: &Ident,
    model: &impl ToTokens,
    fields: &[Ident],
    types: &[Type],
) -> TokenStream {
    let patch_selector = format_ident!("__{strct}_Selector");
    quote! {
        const _: () = {
            use ::rorm::internal::field::AbstractField;

            pub struct #patch_selector<P> {
                #(
                    #fields: ::rorm::crud::selectable::PatchFieldSelector<#types, P>,
                )*
            }
            impl<P> ::rorm::crud::selectable::Selectable for #patch_selector<P>
            where
                P: ::rorm::internal::relation_path::Path
            {
                type Model = P::Origin;

                type Result = #strct;

                fn prepare(&self, context: &mut ::rorm::internal::query_context::QueryContext) {
                    #(
                        self.#fields.prepare(context);
                    )*
                }

                fn decode(&self, row: &::rorm::row::Row) -> Result<Self::Result, ::rorm::Error> {
                    Ok(#strct {#(
                        #fields: self.#fields.decode(row)?,
                    )*})
                }
            }

            impl ::rorm::model::Patch for #strct {
                type Model = #model;

                type Selector<P: ::rorm::internal::relation_path::Path> = #patch_selector<P>;

                fn select<P: ::rorm::internal::relation_path::Path>() -> Self::Selector<P> {
                    #patch_selector {
                        #(
                            #fields: ::rorm::crud::selectable::PatchFieldSelector::new(
                                <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields
                            ),
                        )*
                    }
                }

                const COLUMNS: &'static [&'static str] = ::rorm::concat_columns!(&[#(
                    ::rorm::internal::field::FieldProxy::columns(<<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields),
                )*]);

                fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
                    use ::rorm::internal::field::AbstractField;
                    #(
                        <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.field()
                            .push_ref(&self.#fields, values);
                    )*
                }

                fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
                    #(
                        <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS.#fields.field()
                            .push_value(self.#fields, values);
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

pub fn try_from_row(
    strct: &Ident,
    model: &impl ToTokens,
    fields: &[Ident],
    ignored: &[Ident],
) -> TokenStream {
    quote! {
        const _: () = {
            use ::rorm::internal::field::AbstractField;

            impl ::rorm::row::FromRow for #strct {
                fn from_row(row: ::rorm::row::Row) -> Result<Self, ::rorm::Error> {
                    Ok(#strct {
                        #(
                            #fields: <#model as ::rorm::model::Model>::FIELDS.#fields.field().get_by_name(&row)?,
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
                                let value = <#model as ::rorm::model::Model>::FIELDS.#fields.field().get_by_index(&row, i as usize)?;
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
        };
    }
}
