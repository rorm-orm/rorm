use crate::generate::SliceExt;
use crate::parse::patch::ParsedPatch;
use crate::MacroConfig;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Type, Visibility};

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

    let fields__ident = fields.map_collect(|x| &x.ident);
    let fields__type = fields.map_collect(|x| &x.ty);

    let partial = partially_generate_patch(
        ident,
        model,
        vis,
        &Default::default(),
        &fields__ident,
        &fields__type,
        config,
    );

    quote! {
        #partial

        #(
            impl #rorm_path::model::GetField<#rorm_path::get_field!(#ident, #fields__ident)> for #ident {
                fn get_field(self) -> #fields__type {
                    self.#fields__ident
                }
                fn borrow_field(&self) -> &#fields__type {
                    &self.#fields__ident
                }
                fn borrow_field_mut(&mut self) -> &mut #fields__type {
                    &mut self.#fields__ident
                }
            }
        )*
    }
}

pub fn partially_generate_patch(
    patch: &Ident,
    model: &impl ToTokens, // Ident or Path
    vis: &Visibility,
    generics: &Generics,
    fields__ident: &[&Ident],
    fields__type: &[&Type],
    config: &MacroConfig,
) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

    let value_space_impl = format_ident!("__{patch}_ValueSpaceImpl");
    let value_space_marker_impl = format_ident!("__{patch}_ValueSpaceImplMarker");

    let decoder = format_ident!("__{patch}_Decoder");

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
                #fields__ident: #rorm_path::crud::selector::FieldDecoder<#fields__type>,
            )*
        }

        impl #impl_generics #rorm_path::crud::selector::Selector for #value_space_impl #type_generics #where_clause {
            type Result = #patch #type_generics;
            type Model = #model #type_generics;
            type Decoder = #decoder #type_generics;
            const INSERT_COMPATIBLE: bool = true;
            fn select(self, ctx: &mut #rorm_path::internal::query_context::QueryContext) -> Self::Decoder {
                #decoder {#(
                    #fields__ident: #rorm_path::internal::patch::model_fields::<#model #type_generics>().#fields__ident.select(&mut *ctx),
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

            fn decode<'index>(&'index self, row: &'_ #rorm_path::db::Row) -> Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                Ok(#patch {#(
                    #fields__ident: self.#fields__ident.decode(row)?,
                )*})
            }
        }

        impl #impl_generics #rorm_path::model::Patch for #patch #type_generics #where_clause {
            type Model = #model #type_generics;

            type ValueSpaceImpl = #value_space_impl #type_generics;

            fn push_columns(columns: &mut Vec<#rorm_path::fields::utils::column_name::ColumnName>) {#(
                columns.extend(
                    #rorm_path::fields::proxy::columns(|| #rorm_path::internal::patch::model_fields::<#model #type_generics>().#fields__ident)
                );
            )*}

            fn push_references<'a>(&'a self, values: &mut Vec<#rorm_path::conditions::Value<'a>>) {
                #(
                    values.extend(#rorm_path::fields::traits::FieldType::as_values(&self.#fields__ident));
                )*
            }

            fn push_values(self, values: &mut Vec<#rorm_path::conditions::Value>) {
                #(
                    values.extend(#rorm_path::fields::traits::FieldType::into_values(self.#fields__ident));
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
