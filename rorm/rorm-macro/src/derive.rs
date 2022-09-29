use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::errors::Errors;
use crate::utils::{get_source, iter_rorm_attributes, to_db_name};

/// Used to match over an [`Ident`] in a similar syntax as over [&str]s.
///
/// The first argument is the identifier to match.
/// The last argument is a default match arm (`_ => ..`).
/// In between an arbitrary number of match arms can be passed.
///
/// ```ignore
/// use syn::Ident;
///
/// let ident = Ident::new("some_identifier", proc_macro2::Span::call_site());
/// match_ident!(ident
///     "foo" => println!("The identifier was 'foo'"),
///     "bar" => println!("The identifier was 'bar'"),
///     _ => println!("The identifier was neither 'foo' nor 'bar'")
/// );
/// ```
///
/// Since [proc_macro2] hides the underlying implementation, it is impossible to actually match
/// over the underlying [&str]. So this macro expands into a lot of `if`s and `else`s.
macro_rules! match_ident {
    ($ident:expr, $( $name:literal => $block:expr ),+, _ => $default:expr) => {
        {
            let ident = $ident;
            $(
                if ident == $name {
                    $block
                } else
            )+
            { $default }
        }
    };
}

pub fn db_enum(enm: TokenStream) -> TokenStream {
    let enm = match syn::parse2::<syn::ItemEnum>(enm) {
        Ok(enm) => enm,
        Err(err) => return err.into_compile_error(),
    };

    let errors = Errors::new();
    let mut identifiers = Vec::new();
    let mut literals = Vec::new();
    for variant in enm.variants.iter() {
        if variant.fields.is_empty() {
            let ident = variant.ident.clone();
            let literal = syn::LitStr::new(&variant.ident.to_string(), variant.ident.span());
            identifiers.push(ident);
            literals.push(literal);
        } else {
            errors.push_new(variant.span(), "Variants aren't allowed to contain data");
        }
    }
    let enum_name = &enm.ident;

    quote! {
        impl ::rorm::model::DbEnum for #enum_name {
            fn from_str(string: &str) -> Self {
                use #enum_name::*;
                match string {
                    #(#literals => #identifiers,)*
                    _ => panic!("Unexpected database value"),
                }
            }
            fn to_str(&self) -> &'static str {
                use #enum_name::*;
                match self {
                    #(#identifiers => #literals,)*
                    _ => unreachable!(),
                }
            }
            fn as_choices() -> Vec<String> {
                vec![
                    #(#literals.to_string()),*
                ]
            }

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

    // Static struct containing all model's fields
    let fields_strct = Ident::new(&format!("__{}_Fields", strct.ident), span);

    let mut model_name = strct.ident.to_string();
    model_name.make_ascii_lowercase();
    let model_name = syn::LitStr::new(&model_name, strct.ident.span());
    let model_source = get_source(&strct);

    let mut primary_field: Option<Ident> = None;
    let mut fields_ident = Vec::new();
    let mut fields_value_type = Vec::new();
    let mut fields_struct_type = Vec::new();
    let mut fields_construction = Vec::new();
    for field in strct.fields.iter() {
        if let Some(ParsedField {
            is_primary,
            ident,
            value_type,
            struct_type,
            construction,
        }) = parse_field(field, &errors)
        {
            match (is_primary, primary_field.as_ref()) {
                (true, None) => primary_field = Some(ident.clone()),
                (true, Some(_)) => errors.push_new(
                    field.ident.span(),
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

    // Static reference pointing to Model::get_imr
    let static_get_imr = Ident::new(&format!("__{}_get_imr", strct.ident), span);

    let strct_ident = strct.ident;
    let field_literals = fields_ident
        .iter()
        .map(|ident| syn::LitStr::new(&ident.to_string(), ident.span()));
    let impl_from_row = from_row(
        &strct_ident,
        &strct_ident,
        &fields_ident,
        &fields_value_type,
    );
    let impl_into_column_iter = into_column_iterator(&strct_ident, &fields_ident);
    TokenStream::from(quote! {
        pub struct #fields_strct {
            #(pub #fields_ident: #fields_struct_type),*
        }

        impl ::rorm::model::Model for #strct_ident {
            type Fields = #fields_strct;
            const FIELDS: Self::Fields = #fields_strct {
                #(
                    #fields_ident: #fields_construction,
                )*
            };

            fn table_name() -> &'static str {
                #model_name
            }

            fn get_imr() -> ::rorm::imr::Model {
                ::rorm::imr::Model {
                    name: #model_name.to_string(),
                    fields: vec![#(
                        (&<#strct_ident as ::rorm::model::Model>::FIELDS.#fields_ident).into(),
                    )*],
                    source_defined_at: #model_source,
                }
            }

            fn as_condition(&self) -> ::rorm::conditional::Condition {
                <#strct_ident as ::rorm::model::Model>::FIELDS
                    .#primary_field.equals(self.#primary_field)
            }
        }

        impl ::rorm::model::Patch for #strct_ident {
            type Model = #strct_ident;

            const COLUMNS: &'static [&'static str] = &[#(
                #field_literals,
            )*];
        }

        #impl_from_row
        #impl_into_column_iter

        #[allow(non_snake_case)]
        #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
        #[::rorm::rename_linkme]
        static #static_get_imr: fn() -> ::rorm::imr::Model = <#strct_ident as ::rorm::model::Model>::get_imr;

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
    let mut field_types = Vec::new();
    for field in strct.fields.iter() {
        if let Some(ident) = field.ident.as_ref() {
            field_idents.push(ident.clone());
            field_types.push(field.ty.clone());
        } else {
            errors.push_new(field.span(), "missing field name");
        }
    }

    let field_literals = field_idents
        .iter()
        .map(|ident| syn::LitStr::new(&ident.to_string(), ident.span()));
    let compile_check = Ident::new(
        &format!("__compile_check_{}", strct.ident.to_string()),
        span,
    );
    let patch_ident = strct.ident.clone();
    let impl_from_row = from_row(&patch_ident, &model_path, &field_idents, &field_types);
    let impl_into_column_iter = into_column_iterator(&patch_ident, &field_idents);
    TokenStream::from(quote! {
        #[allow(non_snake_case)]
        fn #compile_check(model: #model_path) {
            // check if the specified model actually implements model
            let _ = <#model_path as ::rorm::model::Model>::table_name();

            // check fields exist on model and match model's types
            // todo error messages for type mismatches are terrible
            let _ = #patch_ident {
                #(
                    #field_idents: model.#field_idents,
                )*
            };
        }

        impl ::rorm::model::Patch for #patch_ident {
            type Model = #model_path;

            const COLUMNS: &'static [&'static str] = &[#(
                #field_literals,
            )*];
        }

        #impl_from_row
        #impl_into_column_iter

        #errors
    })
}

fn from_row(
    strct: &Ident,
    model: &impl ToTokens,
    fields: &[Ident],
    types: &[syn::Type],
) -> TokenStream {
    quote! {
        impl TryFrom<::rorm::row::Row> for #strct {
            type Error = ::rorm::error::Error;

            fn try_from(row: ::rorm::row::Row) -> Result<Self, Self::Error> {
                Ok(#strct {
                    #(
                        #fields: <#types as ::rorm::model::AsDbType>::from_primitive(
                            row.get(<#model as ::rorm::model::Model>::FIELDS.#fields.name)?
                        ),
                    )*
                })
            }
        }
    }
}

fn into_column_iterator(patch: &Ident, columns: &[Ident]) -> TokenStream {
    let iter_ident = quote::format_ident!("__{}Iterator", patch);
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

struct ParsedField {
    is_primary: bool,
    ident: Ident,
    value_type: syn::Type,
    struct_type: TokenStream,
    construction: TokenStream,
}
fn parse_field(field: &syn::Field, errors: &Errors) -> Option<ParsedField> {
    let ident = if let Some(ident) = field.ident.as_ref() {
        ident.clone()
    } else {
        errors.push_new(field.ident.span(), "field has no name");
        return None;
    };

    let mut is_primary = false;
    let mut annotations = Vec::new();
    let mut has_choices = false;
    for meta in iter_rorm_attributes(&field.attrs, &errors) {
        // Get the annotation's identifier.
        // Since one is required for every annotation, error if it is missing.
        let ident = if let Some(ident) = meta.path().get_ident() {
            ident
        } else {
            errors.push_new(meta.path().span(), "expected identifier");
            continue;
        };

        // Parse a simple annotation taking no arguments and simply adding its associated variant
        macro_rules! parse_anno {
            ($name:literal, $variant:literal) => {{
                if let syn::Meta::Path(_) = meta {
                    let variant = Ident::new($variant, ident.span());
                    annotations.push(quote! {
                        ::rorm::model::Annotation::#variant
                    });
                } else {
                    errors.push_new(
                        meta.span(),
                        concat!($name, " doesn't take any values: #[rorm(", $name, ")]"),
                    );
                }
            }};
        }

        match_ident!(ident,
            "auto_create_time" => parse_anno!("auto_create_time", "AutoCreateTime"),
            "auto_update_time" => parse_anno!("auto_update_time", "AutoUpdateTime"),
            "primary_key" => {parse_anno!("primary_key", "PrimaryKey"); is_primary = true;},
            "unique" => parse_anno!("unique", "Unique"),
            "autoincrement" => parse_anno!("autoincrement", "AutoIncrement"),
            "id" => {
                if let syn::Meta::Path(_) = meta {
                    annotations.push(quote! {
                        ::rorm::model::Annotation::PrimaryKey
                    });
                    annotations.push(quote! {
                        ::rorm::model::Annotation::AutoIncrement
                    });
                    is_primary = true;
                } else {
                    errors.push_new(
                        meta.span(),
                        "id doesn't take any values: #[rorm(id)]",
                    );
                }
            },
            "default" => parse_default(&mut annotations, &errors, &meta),
            "max_length" => parse_max_length(&mut annotations, &errors, &meta),
            "choices" => {parse_choices(&mut annotations, &errors, &meta); has_choices = true;},
            "index" => parse_index(&mut annotations, &errors, &meta),
            _ => errors.push_new(ident.span(), "Unknown annotation")
        );
    }

    let value_type = field.ty.clone();

    let db_name = syn::LitStr::new(&to_db_name(ident.to_string()), field.span());

    let db_type = if has_choices {
        quote! { ::rorm::hmr::Choices }
    } else {
        quote! { <#value_type as ::rorm::model::AsDbType>::DbType }
    };

    let source = get_source(&field);

    Some(ParsedField {
        is_primary,
        struct_type: TokenStream::from(quote! {
            ::rorm::model::Field<#value_type, #db_type>
        }),
        construction: TokenStream::from(quote! {
            ::rorm::model::Field {
                name: #db_name,
                annotations: &[
                    #(#annotations),*
                ],
                source: #source,
                _phantom: ::std::marker::PhantomData,
            }
        }),
        ident,
        value_type,
    })
}

/// Parse the `#[rorm(default = ..)]` annotation.
///
/// It accepts a single literal as argument.
/// Currently the only supported literal types are:
/// - String
/// - Integer
/// - Floating Point Number
/// - Boolean
///
/// TODO: Figure out how to check the literal's type is compatible with the annotated field's type
fn parse_default(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    let arg = match meta {
        syn::Meta::NameValue(syn::MetaNameValue { lit, .. }) => lit,
        _ => {
            errors.push_new(
                meta.span(),
                "default expects a single literal: #[rorm(default = ..)]",
            );
            return;
        }
    };

    let variant = match arg {
        syn::Lit::Str(_) => "String",
        syn::Lit::Int(_) => "Integer",
        syn::Lit::Float(_) => "Float",
        syn::Lit::Bool(_) => "Boolean",
        _ => {
            errors.push_new(arg.span(), "unsupported literal");
            return;
        }
    };

    let variant = Ident::new(variant, arg.span());
    annotations.push(TokenStream::from(quote! {
        ::rorm::model::Annotation::DefaultValue(::rorm::model::DefaultValue::#variant(#arg))
    }));
}

/// Parse the `#[rorm(max_length = ..)]` annotation.
///
/// It accepts a single integer literal as argument.
fn parse_max_length(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    match meta {
        syn::Meta::NameValue(syn::MetaNameValue {
            lit: syn::Lit::Int(integer),
            ..
        }) => {
            annotations.push(TokenStream::from(quote! {
                ::rorm::model::Annotation::MaxLength(#integer)
            }));
        }
        _ => {
            errors.push_new(
                meta.span(),
                "max_length expects a single integer literal: #rorm(max_length = 255)",
            );
        }
    }
}

/// Parse the `#[rorm(choices(..))]` annotation.
///
/// It accepts any number of string literals as arguments.
fn parse_choices(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    let usage_string =
        "choices expects a list of string literals: #[rorm(choices(\"foo\", \"bar\", ..))]";

    // Check if used as list i.e. "function call"
    if let syn::Meta::List(syn::MetaList { nested, .. }) = meta {
        let mut choices = Vec::new();

        // Check and collect string literals
        for choice in nested.iter() {
            match choice {
                syn::NestedMeta::Lit(syn::Lit::Str(choice)) => {
                    choices.push(choice);
                }
                _ => {
                    errors.push_new(choice.span(), usage_string);
                    continue;
                }
            }
        }

        annotations.push(TokenStream::from(quote! {
            ::rorm::model::Annotation::Choices(&[
                #(#choices),*
            ])
        }));
    } else {
        errors.push_new(meta.span(), usage_string);
    }
}

/// Parse the `#[rorm(index)]` annotation.
///
/// It accepts four different syntax's:
/// - `#[rorm(index)]`
/// - `#[rorm(index())]`
///    *(semantically identical to first one)*
/// - `#[rorm(name = <string literal>)]`
/// - `#[rorm(name = <string literal>, priority = <integer literal>)]`
///    *(insensitive to argument order)*
fn parse_index(annotations: &mut Vec<TokenStream>, errors: &Errors, meta: &syn::Meta) {
    match &meta {
        // index was used on its own without arguments
        syn::Meta::Path(_) => {
            annotations.push(TokenStream::from(quote! {
                ::rorm::model::Annotation::Index(None)
            }));
        }

        // index was used as "function call"
        syn::Meta::List(syn::MetaList { nested, .. }) => {
            let mut name = None;
            let mut prio = None;

            // Loop over arguments extracting `name` and `prio` while reporting any errors
            for nested_meta in nested.into_iter() {
                // Only accept keyword arguments
                let (path, literal) =
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit,
                        ..
                    })) = &nested_meta
                    {
                        (path.clone(), lit.clone())
                    } else {
                        errors.push_new(
                            nested_meta.span(),
                            "index expects keyword arguments: #[rorm(index(name = \"...\"))]",
                        );
                        continue;
                    };

                // Only accept keywords who are identifier
                let ident = if let Some(ident) = path.get_ident() {
                    ident
                } else {
                    errors.push_new(
                        nested_meta.span(),
                        "index expects keyword arguments: #[rorm(index(name = \"...\"))]",
                    );
                    continue;
                };

                // Only accept "name" and "prio" as keywords
                // Check the associated value's type
                // Report duplications
                if ident == "name" {
                    if name.is_none() {
                        match literal {
                            syn::Lit::Str(literal) => {
                                name = Some(literal);
                            }
                            _ => {
                                errors.push_new(
                                    literal.span(),
                                    "name expects a string literal: #[rorm(index(name = \"...\"))]",
                                );
                            }
                        }
                    } else {
                        errors.push_new(ident.span(), "name has already been set");
                    }
                } else if ident == "priority" {
                    if prio.is_none() {
                        match literal {
                            syn::Lit::Int(literal) => {
                                prio = Some(literal);
                            }
                            _ => {
                                errors.push_new(literal.span(), "priority expects a integer literal: #[rorm(index(priority = \"...\"))]");
                            }
                        };
                    } else {
                        errors.push_new(ident.span(), "priority has already been set");
                    }
                } else {
                    errors.push_new(ident.span(), "unknown keyword argument");
                }
            }

            // Produce output depending on the 4 possible configurations
            // of `prio.is_some()` and `name.is_some()`
            if prio.is_some() && name.is_none() {
                errors.push_new(
                    meta.span(),
                    "index also requires a name when a priority is defined",
                );
            } else {
                let inner = if let Some(name) = name {
                    let prio = if let Some(prio) = prio {
                        quote! { Some(#prio) }
                    } else {
                        quote! { None }
                    };
                    quote! { Some(::rorm::model::IndexValue { name: #name, priority: #prio }) }
                } else {
                    quote! { None }
                };
                annotations.push(TokenStream::from(quote! {
                    ::rorm::model::Annotation::Index(#inner)
                }));
            }
        }

        // index was used as keyword argument
        _ => {
            errors.push_new(meta.span(), "index ether stands on its own or looks like a function call: #[rorm(index)] or #[rorm(index(..))]");
        }
    }
}
