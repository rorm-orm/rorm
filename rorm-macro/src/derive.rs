use darling::{Error, FromAttributes, FromMeta};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{LitStr, Type, Visibility};

use crate::annotations::{Annotations, FieldPath};
use crate::utils::{get_source, to_db_name};
use crate::{annotations, trait_impls};

pub fn db_enum(enm: TokenStream) -> darling::Result<TokenStream> {
    let enm = match syn::parse2::<syn::ItemEnum>(enm) {
        Ok(enm) => enm,
        Err(err) => return Ok(err.into_compile_error()),
    };

    let mut errors = Error::accumulator();
    let mut identifiers = Vec::new();
    for variant in enm.variants {
        if variant.fields.is_empty() {
            identifiers.push(variant.ident);
        } else {
            errors.push(
                Error::unsupported_shape("variants aren't allowed to contain data")
                    .with_span(&variant.fields),
            );
        }
    }
    let db_enum = enm.ident;

    errors.finish()?;
    Ok(quote! {
        const _: () = {
            const CHOICES: &'static [&'static str] = &[
                #(stringify!(#identifiers)),*
            ];

            impl ::rorm::internal::field::as_db_type::AsDbType for #db_enum {
                type Primitive = String;
                type DbType<F: ::rorm::internal::field::Field> = ::rorm::internal::hmr::db_type::Choices;

                const NULL_TYPE: ::rorm::value::NullType = ::rorm::value::NullType::String;

                const IMPLICIT: Option<::rorm::internal::hmr::annotations::Annotations> = Some({
                    let mut annos = ::rorm::internal::hmr::annotations::Annotations::empty();
                    annos.choices = Some(::rorm::internal::hmr::annotations::Choices(CHOICES));
                    annos
                });

                fn from_primitive(primitive: Self::Primitive) -> Self {
                    use #db_enum::*;
                    match primitive.as_str() {
                        #(stringify!(#identifiers) => #identifiers,)*
                        _ => panic!("Unexpected database value"),
                    }
                }

                fn as_primitive<F>(&self) -> ::rorm::conditions::Value {
                    ::rorm::conditions::Value::String(CHOICES[*self as usize])
                }
            }
        };
    })
}

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(rorm), default)]
pub struct ModelAnnotations {
    pub rename: Option<LitStr>,
}

pub fn model(strct: TokenStream) -> darling::Result<TokenStream> {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return Ok(err.into_compile_error()),
    };

    let span = Span::call_site();

    let mut errors = Error::accumulator();
    let annotations = errors
        .handle(ModelAnnotations::from_attributes(&strct.attrs))
        .unwrap_or_default();
    let mut fields = Vec::new();
    let mut ignored_fields = Vec::new();
    for (index, field) in strct.fields.into_iter().enumerate() {
        if let Some(field) = errors.handle(parse_field(index, field, &strct.ident, &strct.vis)) {
            match field {
                ParsedField::Db(field) => fields.push(field),
                ParsedField::Ignored(name) => ignored_fields.push(name),
            }
        }
    }

    let mut primary_field: Option<&Field> = None;
    for field in fields.iter() {
        match (field.is_primary, primary_field) {
            (true, None) => primary_field = Some(field),
            (true, Some(_)) => errors.push(
                Error::custom("Another primary key column has already been defined.")
                    .with_span(&field.ident),
            ),
            _ => {}
        }
    }
    let (primary_struct, primary_ident) = if let Some(primary_field) = primary_field {
        (
            primary_field.type_ident.clone(),
            primary_field.ident.clone(),
        )
    } else {
        errors.push(
            Error::custom("Missing primary key. Please annotate a field with ether `#[rorm(id)]` or `#[rorm(primary_key)]`")
                .with_span(&Span::call_site()),
        );
        (Ident::new("_", span), Ident::new("_", span))
    };

    // Static struct containing all model's fields
    let fields_struct = format_ident!("__{}_Fields_Struct", strct.ident);
    let fields_struct_doc = LitStr::new(
        &format!(
            "[`{}`]'s [`Fields`](::rorm::model::Model::Fields) struct.",
            strct.ident
        ),
        span,
    );
    // Static reference pointing to Model::get_imr
    let static_get_imr = format_ident!("__{}_get_imr", strct.ident);
    // Const name for compile time checks
    let compile_check = format_ident!("__compile_check_{}", strct.ident);

    // Database table's name
    let table_name = annotations
        .rename
        .unwrap_or_else(|| LitStr::new(&to_db_name(strct.ident.to_string()), strct.ident.span()));
    if table_name.value().contains("__") {
        errors.push(Error::custom("Names can't contain a double underscore. You might want to consider using `#[rorm(rename = \"...\")]`.").with_span(&table_name));
    }

    // File, line and column the struct was defined in
    let model_source = get_source(&span);

    let fields_ident = Vec::from_iter(fields.iter().map(|field| field.ident.clone()));
    let vis = strct.vis;
    let model = strct.ident;
    let impl_patch = trait_impls::patch(&model, &model, &fields_ident);
    let impl_try_from_row =
        trait_impls::try_from_row(&model, &model, &fields_ident, &ignored_fields);

    let fields_vis = fields.iter().map(|field| &field.vis);
    let fields_type: Vec<_> = fields.iter().map(|field| &field.type_ident).collect();
    let fields_index = fields.iter().map(|field| &field.index);
    let fields_doc = fields.iter().map(|field| {
        LitStr::new(
            &format!("[`{}`]'s `{}` field", model, field.ident),
            field.ident.span(),
        )
    });

    let non_primary_raw_type = fields
        .iter()
        .filter(|field| !field.is_primary)
        .map(|field| &field.raw_type);
    let non_primary_type = fields
        .iter()
        .filter(|field| !field.is_primary)
        .map(|field| &field.type_ident);
    let non_primary_ident = fields
        .iter()
        .filter(|field| !field.is_primary)
        .map(|field| &field.ident);

    errors.finish()?;
    Ok(quote! {
        #(
            #fields
        )*

        #[doc = #fields_struct_doc]
        #[allow(non_camel_case_types)]
        #vis struct #fields_struct<Path> {
            #(
                #[doc = #fields_doc]
                #fields_vis #fields_ident: ::rorm::internal::field::FieldProxy<#fields_type, Path>,
            )*
        }
        impl<Path> ::rorm::model::ConstNew for #fields_struct<Path> {
            const NEW: Self = Self {
                #(
                    #fields_ident: ::rorm::internal::field::FieldProxy::new(),
                )*
            };
        }

        impl ::rorm::model::Model for #model {
            type Primary = #primary_struct;

            type Fields<P: ::rorm::internal::relation_path::Path> = #fields_struct<P>;

            const TABLE: &'static str = #table_name;

            fn get_imr() -> ::rorm::imr::Model {
                ::rorm::imr::Model {
                    name: #table_name.to_string(),
                    fields: [#(
                        <#fields_type as ::rorm::internal::field::AbstractField<_>>::imr(),
                    )*].into_iter().flatten().collect(),
                    source_defined_at: #model_source,
                }
            }
        }

        #(
            impl ::rorm::model::FieldByIndex<{ #fields_index }> for #model {
                type Field = #fields_type;
            }
        )*

        #(

            impl ::rorm::model::UpdateField<#non_primary_type> for #model {
                fn update_field<'m, T>(
                    &'m mut self,
                    update: impl FnOnce(&'m <#primary_struct as ::rorm::internal::field::Field>::Type, &'m mut #non_primary_raw_type) -> T,
                ) -> T {
                    update(&self.#primary_ident, &mut self.#non_primary_ident)
                }
            }
        )*

        #[allow(non_upper_case_globals)]
        const #compile_check: () = {
            // Cross field checks
            let mut count_auto_increment = 0;
            #(
                if let Some(annos) = <#fields_type as ::rorm::internal::field::AbstractField>::DB_ANNOTATIONS {
                    if annos.auto_increment.is_some() {
                        count_auto_increment += 1;
                    }
                }
            )*
            if count_auto_increment > 1 {
                panic!("\"auto_increment\" can only be set once per model");
            }
        };

        #impl_patch
        #impl_try_from_row

        #[allow(non_upper_case_globals)]
        #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
        #[::rorm::rename_linkme]
        static #static_get_imr: fn() -> ::rorm::imr::Model = <#model as ::rorm::model::Model>::get_imr;
    })
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct PatchAnnotations {
    pub model: ModelPath,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct NoAnnotations;

#[derive(Debug)]
pub struct ModelPath(syn::Path);
impl FromMeta for ModelPath {
    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<syn::Path>(value)
            .map(ModelPath)
            .map_err(|error| Error::unknown_value(&error.to_string()))
    }
}

pub fn patch(strct: TokenStream) -> darling::Result<TokenStream> {
    let strct = match syn::parse2::<syn::ItemStruct>(strct) {
        Ok(strct) => strct,
        Err(err) => return Ok(err.into_compile_error()),
    };

    let mut errors = Error::accumulator();

    let mut field_idents = Vec::new();
    for field in strct.fields {
        errors.handle(NoAnnotations::from_attributes(&field.attrs));
        if let Some(ident) = field.ident {
            field_idents.push(ident);
        } else {
            errors.push(Error::custom("missing field name").with_span(&field));
        }
    }

    let Some(PatchAnnotations {model: ModelPath(model_path)}) = errors.handle(PatchAnnotations::from_attributes(&strct.attrs)) else {
        return errors.finish_with(TokenStream::new());
    };

    let patch = strct.ident;
    let compile_check = format_ident!("__compile_check_{}", patch);
    let impl_patch = trait_impls::patch(&patch, &model_path, &field_idents);
    let impl_try_from_row = trait_impls::try_from_row(&patch, &model_path, &field_idents, &[]);

    errors.finish()?;
    Ok(quote! {
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
    })
}

enum ParsedField {
    Db(Field),
    Ignored(Ident),
}
fn parse_field(
    index: usize,
    field: syn::Field,
    model: &Ident,
    model_vis: &Visibility,
) -> darling::Result<ParsedField> {
    let mut errors = Error::accumulator();

    let ident = if let Some(ident) = field.ident {
        ident
    } else {
        errors.push(Error::unsupported_shape("field has no name").with_span(&field.ident));
        Ident::new("_", field.span())
    };

    let mut annotations = errors
        .handle(annotations::Annotations::from_attributes(&field.attrs))
        .unwrap_or_default();

    if annotations.ignore {
        return errors.finish_with(ParsedField::Ignored(ident));
    }

    let db_name = annotations
        .rename
        .take()
        .unwrap_or_else(|| LitStr::new(&to_db_name(ident.to_string()), ident.span()));
    if db_name.value().contains("__") {
        errors.push(Error::custom("Names can't contain a double underscore. You might want to consider using `#[rorm(rename = \"...\")]`.").with_span(&db_name));
    }

    errors.finish()?;

    let is_primary = annotations.primary_key || annotations.id;

    Ok(ParsedField::Db(Field {
        model_vis: model_vis.clone(),
        type_ident: format_ident!("__{}_{}", model, ident),
        raw_type: field.ty,
        related_field: annotations.field.take(),
        model: model.clone(),
        index: Literal::usize_unsuffixed(index),
        db_name,
        annotations,

        is_primary,
        ident,
        vis: if is_primary {
            model_vis.clone()
        } else {
            field.vis
        },
    }))
}

struct Field {
    model_vis: Visibility,
    type_ident: Ident,
    raw_type: Type,
    related_field: Option<FieldPath>,
    model: Ident,
    index: Literal,
    db_name: LitStr,
    annotations: Annotations,

    is_primary: bool,
    ident: Ident,
    vis: Visibility,
}
impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            model_vis,
            type_ident,
            raw_type,
            related_field,
            model,
            index,
            db_name,
            annotations,
            ident,
            ..
        } = self;

        let db_type = if annotations.choices.is_some() {
            quote! { ::rorm::internal::hmr::db_type::Choices }
        } else {
            quote! { () }
        };

        let related_field = if let Some(FieldPath { model, field, span }) = related_field.as_ref() {
            quote_spanned! {*span=> <#model as ::rorm::model::FieldByIndex<{<#model as ::rorm::model::Model>::F.#field.index()}>>::Field}
        } else {
            quote! { ::rorm::internal::field::MissingRelatedField }
        };

        let source = get_source(&ident);

        let doc = LitStr::new(
            &format!("rorm's representation of [`{}`]'s `{}` field", model, ident),
            ident.span(),
        );

        let temp = quote! {
            #[doc = #doc]
            #[allow(non_camel_case_types)]
            #model_vis struct #type_ident;
            impl ::rorm::internal::field::RawField for #type_ident {
                type Kind = <#raw_type as ::rorm::internal::field::FieldType>::Kind;
                type RawType = #raw_type;
                type ExplicitDbType = #db_type;
                type RelatedField = #related_field;
                type Model = #model;
                const INDEX: usize = #index;
                const NAME: &'static str = #db_name;
                const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = #annotations;
                const SOURCE: Option<::rorm::internal::hmr::Source> = #source;
            }
        };
        tokens.extend(temp);
    }
}
