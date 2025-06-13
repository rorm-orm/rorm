use std::array;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Type, Visibility};

use crate::parse::patch::ParsedPatch;
use crate::MacroConfig;

pub fn generate_patch(patch: &ParsedPatch, config: &MacroConfig) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

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
        &Default::default(),
        field_idents_1.clone(),
        fields.iter().map(|field| &field.ty),
        config,
    );

    quote! {
        #partial

        #(
            impl #rorm_path::model::GetField<#rorm_path::get_field!(#ident, #field_idents_2)> for #ident {
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
    }
}

pub fn partially_generate_patch<'a>(
    patch: &Ident,
    model: &impl ToTokens, // Ident or Path
    vis: &Visibility,
    generics: &Generics,
    fields: impl Iterator<Item = &'a Ident> + Clone,
    types: impl Iterator<Item = &'a Type> + Clone,
    config: &MacroConfig,
) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

    let value_space_impl = format_ident!("__{patch}_ValueSpaceImpl");
    let value_space_marker_impl = format_ident!("__{patch}_ValueSpaceImplMarker");

    let decoder = format_ident!("__{patch}_Decoder");

    let [fields_1, fields_2, fields_3, fields_4, fields_5, fields_6, fields_7] =
        array::from_fn(|_| fields.clone());
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let lifetime_generics = {
        let mut tokens = impl_generics
            .to_token_stream()
            .into_iter()
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            quote! {<'a>}
        } else {
            tokens.remove(0);
            tokens.pop();
            quote! {<'a, #(#tokens)*>}
        }
    };
    quote! {
        // Credit and explanation: https://github.com/dtolnay/case-studies/tree/master/unit-type-parameters
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis enum #value_space_impl #impl_generics #where_clause {
            #patch,

            #[allow(dead_code)]
            #[doc(hidden)]
            #value_space_marker_impl(::std::marker::PhantomData<#patch #type_generics>),
        }
        #vis use #value_space_impl::*;

        #[doc(hidden)]
        #vis struct #decoder #impl_generics #where_clause {
            #(
                #fields_1: <#types as #rorm_path::fields::traits::FieldType>::Decoder,
            )*
        }

        impl #impl_generics #rorm_path::crud::selector::Selector for #value_space_impl #type_generics #where_clause {
            type Result = #patch #type_generics;
            type Model = #model #type_generics;
            type Decoder = #decoder #type_generics;
            const INSERT_COMPATIBLE: bool = true;
            fn select(self, ctx: &mut #rorm_path::internal::query_context::QueryContext) -> Self::Decoder {
                #decoder {#(
                    #fields_4: #rorm_path::internal::patch::model_fields::<#model #type_generics>().#fields_4.select(&mut *ctx),
                )*}
            }
        }

        impl #impl_generics ::std::default::Default for #value_space_impl #type_generics #where_clause {
            fn default() -> Self {
                Self::#patch
            }
        }

        impl #impl_generics #rorm_path::crud::decoder::Decoder for #decoder #type_generics #where_clause {
            type Result = #patch #type_generics;

            fn by_name<'index>(&'index self, row: &'_ #rorm_path::db::Row) -> Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                Ok(#patch {#(
                    #fields_2: self.#fields_2.by_name(row)?,
                )*})
            }

            fn by_index<'index>(&'index self, row: &'_ #rorm_path::db::Row) -> Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                Ok(#patch {#(
                    #fields_3: self.#fields_3.by_index(row)?,
                )*})
            }
        }

        impl #impl_generics #rorm_path::model::Patch for #patch #type_generics #where_clause {
            type Model = #model #type_generics;

            type ValueSpaceImpl = #value_space_impl #type_generics;

            fn push_columns(columns: &mut Vec<#rorm_path::fields::utils::column_name::ColumnName>) {#(
                columns.extend(
                    #rorm_path::fields::proxy::columns(|| #rorm_path::internal::patch::model_fields::<#model #type_generics>().#fields_5)
                );
            )*}

            fn push_references<'a>(&'a self, values: &mut Vec<#rorm_path::conditions::Value<'a>>) {
                #(
                    values.extend(#rorm_path::fields::traits::FieldType::as_values(&self.#fields_6));
                )*
            }

            fn push_values(self, values: &mut Vec<#rorm_path::conditions::Value>) {
                #(
                    values.extend(#rorm_path::fields::traits::FieldType::into_values(self.#fields_7));
                )*
            }
        }

        impl #lifetime_generics #rorm_path::internal::patch::IntoPatchCow<'a> for #patch #type_generics #where_clause {
            type Patch = #patch #type_generics;

            fn into_patch_cow(self) -> #rorm_path::internal::patch::PatchCow<'a, #patch #type_generics> {
                #rorm_path::internal::patch::PatchCow::Owned(self)
            }
        }
        impl #lifetime_generics #rorm_path::internal::patch::IntoPatchCow<'a> for &'a #patch #type_generics #where_clause {
            type Patch = #patch #type_generics;

            fn into_patch_cow(self) -> #rorm_path::internal::patch::PatchCow<'a, #patch #type_generics> {
                #rorm_path::internal::patch::PatchCow::Borrowed(self)
            }
        }
    }
}
