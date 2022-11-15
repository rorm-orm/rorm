use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;

use crate::errors::Errors;
use crate::utils::{get_source, iter_rorm_attributes, to_db_name};
use crate::{annotations, trait_impls};

pub fn db_enum(enm: TokenStream) -> TokenStream {
    let enm = match syn::parse2::<syn::ItemEnum>(enm) {
        Ok(enm) => enm,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let mut identifiers = Vec::new();
    for variant in enm.variants {
        if variant.fields.is_empty() {
            identifiers.push(variant.ident);
        } else {
            errors.push_new(variant.span(), "Variants aren't allowed to contain data");
        }
    }
    let db_enum = enm.ident;

    quote! {
        impl ::rorm::model::DbEnum for #db_enum {
            fn from_str(string: &str) -> Self {
                use #db_enum::*;
                match string {
                    #(stringify!(#identifiers) => #identifiers,)*
                    _ => panic!("Unexpected database value"),
                }
            }
            fn to_str(&self) -> &'static str {
                Self::CHOICES[*self as usize]
            }
            const CHOICES: &'static [&'static str] = &[
                #(stringify!(#identifiers)),*
            ];

            #errors
        }
    }
}

pub fn model(strct: TokenStream) -> TokenStream {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let span = proc_macro2::Span::call_site();

    let mut darling_acc = darling::error::Accumulator::default();
    let mut fields = Vec::with_capacity(strct.fields.len());
    for (index, field) in strct.fields.into_iter().enumerate() {
        if let Some(Some(field)) =
            darling_acc.handle(parse_field(index, field, &strct.ident, &strct.vis, &errors))
        {
            fields.push(field);
        }
    }
    let darling_err = darling_acc
        .finish()
        .err()
        .map(darling::error::Error::write_errors);

    let mut primary_field: Option<&ParsedField> = None;
    for field in fields.iter() {
        match (field.is_primary, primary_field) {
            (true, None) => primary_field = Some(field),
            (true, Some(_)) => errors.push_new(
                field.ident.span(),
                "Another primary key column has already been defined.",
            ),
            _ => {}
        }
    }
    let primary_field = if let Some(primary_field) = primary_field {
        primary_field
    } else {
        errors.push_new(
            span,
            "Missing primary key. Please annotate a field with ether `#[rorm(id)]` or `#[rorm(primary_key)]`",
        );
        return errors.into_token_stream();
    };

    // Static struct containing all model's fields
    let fields_struct = format_ident!("__{}_Fields_Struct", strct.ident);
    // Static reference pointing to Model::get_imr
    let static_get_imr = format_ident!("__{}_get_imr", strct.ident);
    // Const name for compile time checks
    let compile_check = format_ident!("__compile_check_{}", strct.ident);
    // Database table's name
    let table_name = syn::LitStr::new(&to_db_name(strct.ident.to_string()), strct.ident.span());
    // File, line and column the struct was defined in
    let model_source = get_source(&span);

    let fields_ident = Vec::from_iter(fields.iter().map(|field| field.ident.clone()));
    let vis = strct.vis;
    let model = strct.ident;
    let impl_patch = trait_impls::patch(&model, &model, &fields_ident);
    let impl_try_from_row = trait_impls::try_from_row(&model, &model, &fields_ident);

    let fields_vis = fields.iter().map(|field| &field.vis);
    let fields_type: Vec<_> = fields.iter().map(|field| &field.type_ident).collect();
    let fields_definition = fields.iter().map(|field| &field.definition);
    let primary_key = &primary_field.type_ident;
    quote! {
        #(
            #[allow(non_camel_case_types)]
            #fields_definition
        )*

        #[allow(non_camel_case_types)]
        #vis struct #fields_struct<Path> {
            #(#fields_vis #fields_ident: ::rorm::internal::field::FieldProxy<#fields_type, Path>),*
        }
        impl<Path> ::rorm::model::ConstNew for #fields_struct<Path> {
            const NEW: Self = Self {
                #(
                    #fields_ident: ::rorm::internal::field::FieldProxy::new(#fields_type),
                )*
            };
        }

        impl ::rorm::model::Model for #model {
            type Primary = #primary_key;

            type Fields<Path> = #fields_struct<Path>;

            const TABLE: &'static str = #table_name;

            fn get_imr() -> ::rorm::imr::Model {
                ::rorm::imr::Model {
                    name: #table_name.to_string(),
                    fields: vec![#(
                        ::rorm::internal::field::as_imr::<#fields_type>(),
                    )*],
                    source_defined_at: #model_source,
                }
            }
        }

        #[allow(non_upper_case_globals)]
        const #compile_check: () = {
            // Cross field checks
            let mut count_auto_increment = 0;
            #(
                let field = <#model as ::rorm::model::Model>::FIELDS.#fields_ident;
                let annos = &field.annotations();
                if annos.auto_increment.is_some() {
                    count_auto_increment += 1;
                }
            )*
            if count_auto_increment > 1 {
                panic!("\"auto_increment\" can only be set once per model");
            }

            ()
        };

        #impl_patch
        #impl_try_from_row

        #[allow(non_upper_case_globals)]
        #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
        #[::rorm::rename_linkme]
        static #static_get_imr: fn() -> ::rorm::imr::Model = <#model as ::rorm::model::Model>::get_imr;

        #errors
        #darling_err
    }
}

pub fn patch(strct: TokenStream) -> TokenStream {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let span = proc_macro2::Span::call_site();

    let mut model_path = None;
    for meta in iter_rorm_attributes(&strct.attrs, &errors) {
        // get the annotation's identifier.
        // since one is required for every annotation, error if it is missing.
        let ident = if let Some(ident) = meta.path().get_ident() {
            ident
        } else {
            errors.push_new(meta.path().span(), "expected identifier");
            continue;
        };

        if ident == "model" {
            if model_path.is_some() {
                errors.push_new(meta.span(), "model is already defined");
                continue;
            }
            match meta {
                syn::Meta::NameValue(value) => match value.lit {
                    syn::Lit::Str(string) => match syn::parse_str::<syn::Path>(&string.value()) {
                        Ok(path) => {
                            model_path = Some(path);
                        }
                        Err(error) => errors.push(error),
                    },
                    _ => errors.push_new(value.lit.span(), "the model attribute expects a path inside a string: `#[rorm(model = \"path::to::model\")]`"),
                }
                _ => errors.push_new(meta.span(), "the model attribute expects a single value: `#[rorm(model = \"path::to::model\")]`"),
            }
        }
    }
    let model_path = if let Some(model_path) = model_path {
        model_path
    } else {
        errors.push_new(span, "missing model attribute. please add `#[rorm(model = \"path::to::model\")]` to your struct!\n\nif you have, maybe you forget to quotes?");
        return errors.into_token_stream();
    };

    let mut field_idents = Vec::new();
    for field in strct.fields {
        for meta in iter_rorm_attributes(&field.attrs, &errors) {
            errors.push_new(meta.span(), "patches don't accept attributes on fields");
        }
        if let Some(ident) = field.ident {
            field_idents.push(ident);
        } else {
            errors.push_new(field.span(), "missing field name");
        }
    }

    let patch = strct.ident;
    let compile_check = format_ident!("__compile_check_{}", patch);
    let impl_patch = trait_impls::patch(&patch, &model_path, &field_idents);
    let impl_try_from_row = trait_impls::try_from_row(&patch, &model_path, &field_idents);
    quote! {
        #[allow(non_snake_case)]
        fn #compile_check(model: #model_path) {
            // check fields exist on model and match model's types
            // todo error messages for type mismatches are terrible
            let _ = #patch {
                #(
                    #field_idents: model.#field_idents,
                )*
            };
        }

        #impl_patch
        #impl_try_from_row

        #errors
    }
}

struct ParsedField {
    is_primary: bool,
    vis: syn::Visibility,
    ident: Ident,
    type_ident: Ident,
    definition: TokenStream,
}
fn parse_field(
    index: usize,
    field: syn::Field,
    model_type: &Ident,
    model_vis: &syn::Visibility,
    errors: &Errors,
) -> darling::Result<Option<ParsedField>> {
    let ident = if let Some(ident) = field.ident {
        ident
    } else {
        errors.push_new(field.ident.span(), "field has no name");
        return Ok(None);
    };

    let annotations = annotations::Annotations::from_attributes(&field.attrs)?;

    let value_type = field.ty;

    let db_name = syn::LitStr::new(&to_db_name(ident.to_string()), ident.span());
    let index = syn::LitInt::new(&index.to_string(), ident.span());

    let db_type = if annotations.choices.is_some() {
        quote! { ::rorm::hmr::db_type::Choices }
    } else {
        quote! { <#value_type as ::rorm::internal::as_db_type::AsDbType>::DbType }
    };

    let is_primary = annotations.primary_key || annotations.id;
    let vis = if is_primary {
        model_vis.clone()
    } else {
        field.vis
    };

    let source = get_source(&ident);

    let type_ident = format_ident!("__{}_{}", model_type, ident);
    let annotations = annotations.to_tokens(&errors);
    let definition = quote! {
        #vis struct #type_ident;
        impl ::rorm::internal::field::Field for #type_ident {
            type Type = #value_type;
            type DbType = #db_type;
            type Model = #model_type;
            const INDEX: usize = #index;
            const NAME: &'static str = #db_name;
            const EXPLICIT_ANNOTATIONS: ::rorm::annotations::Annotations = #annotations;
            const SOURCE: Option<::rorm::hmr::Source> = #source;
        }
    };
    Ok(Some(ParsedField {
        is_primary,
        vis,
        ident,
        type_ident,
        definition,
    }))
}
