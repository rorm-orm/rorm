use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;

use crate::annotations::{Annotation, Annotations};
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

    let mut primary_field: Option<Ident> = None;
    let mut fields_ident = Vec::new();
    let mut fields_value_type = Vec::new();
    let mut fields_struct_type = Vec::new();
    let mut fields_construction = Vec::new();
    for (index, field) in strct.fields.into_iter().enumerate() {
        if let Some(ParsedField {
            is_primary,
            ident,
            value_type,
            struct_type,
            construction,
        }) = parse_field(index, field, &errors)
        {
            match (is_primary, primary_field.as_ref()) {
                (true, None) => primary_field = Some(ident.clone()),
                (true, Some(_)) => errors.push_new(
                    ident.span(),
                    "Another primary key column has already been defined.",
                ),
                _ => {}
            }
            fields_ident.push(ident);
            fields_value_type.push(value_type);
            fields_struct_type.push(struct_type);
            fields_construction.push(construction);
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

    let model = strct.ident;
    let impl_patch = trait_impls::patch(&model, &model, &fields_ident);
    let impl_try_from_row = trait_impls::try_from_row(&model, &model, &fields_ident);
    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        pub struct #fields_struct {
            #(pub #fields_ident: #fields_struct_type),*
        }

        impl ::rorm::model::Model for #model {
            const PRIMARY: (&'static str, usize) = (Self::FIELDS.#primary_field.name, Self::FIELDS.#primary_field.index);

            type Fields = #fields_struct;
            const F: Self::Fields = #fields_struct {
                #(
                    #fields_ident: #fields_construction,
                )*
            };

            fn table_name() -> &'static str {
                #table_name
            }

            fn get_imr() -> ::rorm::imr::Model {
                ::rorm::imr::Model {
                    name: #table_name.to_string(),
                    fields: vec![#(
                        (&<#model as ::rorm::model::Model>::FIELDS.#fields_ident).into(),
                    )*],
                    source_defined_at: #model_source,
                }
            }
        }

        #[allow(non_upper_case_globals)]
        const #compile_check: () = {
            #(
                {const _CHECK: () = <#model as ::rorm::model::Model>::FIELDS.#fields_ident.check_annotations();}
            )*

            // Cross field checks
            let mut count_auto_increment = 0;
            #(
                let field = <#model as ::rorm::model::Model>::FIELDS.#fields_ident;
                let annos = &field.annotations;
                if annos.is_auto_increment_set() {
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
    })
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
    TokenStream::from(quote! {
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
    })
}

struct ParsedField {
    is_primary: bool,
    ident: Ident,
    value_type: syn::Type,
    struct_type: TokenStream,
    construction: TokenStream,
}
fn parse_field(index: usize, field: syn::Field, errors: &Errors) -> Option<ParsedField> {
    let ident = if let Some(ident) = field.ident {
        ident
    } else {
        errors.push_new(field.ident.span(), "field has no name");
        return None;
    };

    let mut annotations = Annotations::new();
    for meta in iter_rorm_attributes(&field.attrs, &errors) {
        // Get the annotation's identifier.
        // Since one is required for every annotation, error if it is missing.
        let ident = if let Some(ident) = meta.path().get_ident() {
            ident
        } else {
            errors.push_new(meta.path().span(), "expected identifier");
            continue;
        };

        for &annotation in annotations::ANNOTATIONS {
            if annotation.applies(ident) {
                annotation.parse(ident, &meta, &mut annotations, &errors);
                break;
            }
        }
    }

    let mut has_choices = false;
    let mut is_primary = false;
    for annotation in annotations.iter() {
        match annotation.annotation {
            Annotation::PrimaryKey => is_primary = true,
            Annotation::Choices => has_choices = true,
            _ => {}
        }
    }

    let value_type = field.ty;

    let db_name = syn::LitStr::new(&to_db_name(ident.to_string()), ident.span());
    let index = syn::LitInt::new(&index.to_string(), ident.span());

    let db_type = if has_choices {
        quote! { ::rorm::hmr::Choices }
    } else {
        quote! { <#value_type as ::rorm::model::AsDbType>::DbType }
    };

    let source = get_source(&ident);

    let builder_steps = annotations.iter_steps();
    let anno_type = annotations.get_type(&value_type);

    Some(ParsedField {
        is_primary,
        struct_type: TokenStream::from(quote! {
            ::rorm::model::Field<
                #value_type,
                #db_type,
                #anno_type,
            >
        }),
        construction: TokenStream::from(quote! {
            ::rorm::model::Field {
                index: #index,
                name: #db_name,
                annotations: <#value_type as ::rorm::model::AsDbType>::ANNOTATIONS #(#builder_steps)* ,
                source: #source,
                _phantom: ::std::marker::PhantomData,
            }
        }),
        ident,
        value_type,
    })
}
