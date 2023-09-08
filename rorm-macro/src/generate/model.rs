use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::LitStr;

use crate::analyze::model::{AnalyzedField, AnalyzedModel, AnalyzedModelFieldAnnotations};
use crate::generate::patch::partially_generate_patch;
use crate::parse::annotations::{Default, Index, NamedIndex, OnAction};
use crate::utils::get_source;

pub fn generate_model(model: &AnalyzedModel) -> TokenStream {
    let fields_struct = generate_fields_struct(model);
    let fields_struct_ident = format_ident!("__{}_Fields_Struct", model.ident);
    let field_declarations = generate_fields(model);
    let AnalyzedModel {
        vis,
        ident,
        table,
        fields,
        primary_key,
    } = model;
    let primary_struct = &fields[*primary_key].unit;
    let primary_ident = &fields[*primary_key].ident;
    let primary_type = &fields[*primary_key].ty;
    let impl_patch = partially_generate_patch(
        ident,
        ident,
        vis,
        fields.iter().map(|field| &field.ident),
        fields.iter().map(|field| &field.ty),
    );
    let field_types = fields.iter().map(|field| &field.ty);
    let field_structs_1 = fields.iter().map(|field| &field.unit);
    let field_structs_2 = field_structs_1.clone();

    let source = get_source(ident);

    let mut tokens = quote! {
        #field_declarations
        #fields_struct

        impl ::rorm::model::Model for #ident {
            type Primary = #primary_struct;

            type Fields<P: ::rorm::internal::relation_path::Path> = #fields_struct_ident<P>;
            const F: #fields_struct_ident<#ident> = ::rorm::model::ConstNew::NEW;
            const FIELDS: #fields_struct_ident<#ident> = ::rorm::model::ConstNew::NEW;

            const TABLE: &'static str = #table;

            fn get_imr() -> ::rorm::imr::Model {
                use ::rorm::internal::field::RawField;
                let mut fields = Vec::new();
                #(
                    fields.extend(<#field_types as ::rorm::fields::traits::FieldType>::get_imr::<#field_structs_1>());
                )*
                ::rorm::imr::Model {
                    name: Self::TABLE.to_string(),
                    fields,
                    source_defined_at: #source,
                }
            }
        }


        const _: () = {
            #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
            #[::rorm::rename_linkme]
            static __get_imr: fn() -> ::rorm::imr::Model = <#ident as ::rorm::model::Model>::get_imr;

            #impl_patch

            // Cross field checks
            let mut count_auto_increment = 0;
            #(
                let annos = <#field_structs_2 as ::rorm::internal::field::RawField>::EFFECTIVE_ANNOTATIONS;
                if let Some(annos) = annos {
                    if annos.auto_increment.is_some() {
                        count_auto_increment += 1;
                    }
                }
            )*
            assert!(count_auto_increment <= 1, "\"auto_increment\" can only be set once per model");
        };
    };
    for (index, field) in fields.iter().enumerate() {
        let field_struct = &field.unit;
        let field_ident = &field.ident;
        let field_type = &field.ty;
        tokens.extend(quote! {
            impl ::rorm::model::FieldByIndex<{ #index }> for #ident {
                type Field = #field_struct;
            }

            impl ::rorm::model::GetField<#field_struct> for #ident {
                fn get_field(self) -> #field_type {
                    self.#field_ident
                }
                fn borrow_field(&self) -> &#field_type {
                    &self.#field_ident
                }
                fn borrow_field_mut(&mut self) -> &mut #field_type {
                    &mut self.#field_ident
                }
            }
        });
        if !field.annos.primary_key {
            tokens.extend(quote! {
                impl ::rorm::model::UpdateField<#field_struct> for #ident {
                    fn update_field<'m, T>(
                        &'m mut self,
                        update: impl FnOnce(&'m #primary_type, &'m mut #field_type) -> T,
                    ) -> T {
                        update(&self.#primary_ident, &mut self.#field_ident)
                    }
                }
            });
        }
    }
    tokens
}

fn generate_fields(model: &AnalyzedModel) -> TokenStream {
    let mut tokens = TokenStream::new();
    let model_ident = &model.ident;
    for (index, field) in model.fields.iter().enumerate() {
        let AnalyzedField {
            vis: _,
            ident,
            column,
            unit,
            ty,
            annos,
        } = field;

        let source = get_source(&ident);
        let vis = &model.vis;
        let doc = LitStr::new(
            &format!("rorm's representation of [`{model_ident}`]'s `{ident}` field",),
            ident.span(),
        );
        let annos = generate_field_annotations(annos);

        tokens.extend(quote_spanned!{ident.span()=>
            #[doc = #doc]
            #[allow(non_camel_case_types)]
            #vis struct #unit;
            impl ::std::clone::Clone for #unit {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl ::std::marker::Copy for #unit {}
            impl ::rorm::internal::field::RawField for #unit {
                type Type = #ty;
                type Model = #model_ident;
                const INDEX: usize = #index;
                const NAME: &'static str = #column;
                const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = #annos;
                const SOURCE: Option<::rorm::internal::hmr::Source> = #source;
                fn new() -> Self {
                    Self
                }
            }
            const _: () = {
                if let Err(err) = <#unit as ::rorm::internal::field::RawField>::CHECK {
                    panic!("{}", err.as_str());
                }
            };
        });
    }
    tokens
}

fn generate_field_annotations(annos: &AnalyzedModelFieldAnnotations) -> TokenStream {
    let AnalyzedModelFieldAnnotations {
        auto_create_time,
        auto_update_time,
        auto_increment,
        primary_key,
        unique,
        on_delete,
        on_update,
        default,
        max_length,
        index,
    } = annos;

    // Convert every field into its "creation" expression
    let auto_create_time = auto_create_time.then(|| quote! {AutoCreateTime});
    let auto_update_time = auto_update_time.then(|| quote! {AutoUpdateTime});
    let auto_increment = auto_increment.then(|| quote! {AutoIncrement});
    let primary_key = primary_key.then(|| quote! {PrimaryKey});
    let unique = unique.then(|| quote! {Unique});
    let max_length = max_length.as_ref().map(|len| quote! {MaxLength(#len)});
    let default = default.as_ref().map(|Default { variant, literal }| {
        let variant = Ident::new(variant, literal.span());
        quote! {DefaultValue(::rorm::internal::hmr::annotations::DefaultValueData::#variant(#literal))}
    });
    let index = index.as_ref().map(|Index(index)| {
        match index {
            None => {
                quote! {Index(None)}
            }

            Some(NamedIndex {
                     name,
                     priority: None,
                 }) => {
                quote! { Index(Some(::rorm::internal::hmr::annotations::IndexData { name: #name, priority: None })) }
            }

            Some(NamedIndex {
                     name,
                     priority: Some(priority),
                 }) => {
                quote! { Index(Some(::rorm::internal::hmr::annotations::IndexData { name: #name, priority: Some(#priority) })) }
            }
        }
    });
    let on_delete = on_delete
        .as_ref()
        .map(|OnAction(token)| quote! {OnDelete::#token});
    let on_update = on_update
        .as_ref()
        .map(|OnAction(token)| quote! {OnUpdate::#token});

    // Unwrap all options
    // Add absolute path
    let finalize = |token: Option<TokenStream>| {
        if let Some(token) = token {
            quote! {Some(::rorm::internal::hmr::annotations::#token)}
        } else {
            quote! {None}
        }
    };
    let auto_create_time = finalize(auto_create_time);
    let auto_update_time = finalize(auto_update_time);
    let auto_increment = finalize(auto_increment);
    let default = finalize(default);
    let index = finalize(index);
    let max_length = finalize(max_length);
    let on_delete = finalize(on_delete);
    let on_update = finalize(on_update);
    let primary_key = finalize(primary_key);
    let unique = finalize(unique);

    quote! {
        ::rorm::internal::hmr::annotations::Annotations {
            auto_create_time: #auto_create_time,
            auto_update_time: #auto_update_time,
            auto_increment: #auto_increment,
            choices: None, // Set implicitly by type
            default: #default,
            index: #index,
            max_length: #max_length,
            on_delete: #on_delete,
            on_update: #on_update,
            primary_key: #primary_key,
            unique: #unique,
            nullable: false, // Set implicitly by type
            foreign: None,   //
        }
    }
}

fn generate_fields_struct(model: &AnalyzedModel) -> TokenStream {
    let vis = &model.vis;
    let ident = format_ident!("__{}_Fields_Struct", model.ident);
    let doc = LitStr::new(
        &format!(
            "[`{}`]'s [`Fields`](::rorm::model::Model::Fields) struct.",
            model.ident
        ),
        Span::call_site(),
    );

    let fields_vis = model.fields.iter().map(|field| &field.vis);
    let fields_doc = model.fields.iter().map(|field| {
        LitStr::new(
            &format!("[`{}`]'s `{}` field", model.ident, field.ident),
            field.ident.span(),
        )
    });
    let fields_ident_1 = model.fields.iter().map(|field| &field.ident);
    let fields_ident_2 = fields_ident_1.clone();
    let fields_type = model.fields.iter().map(|field| &field.unit);

    quote! {
        #[doc = #doc]
        #[allow(non_camel_case_types)]
        #vis struct #ident<Path> {
            #(
                #[doc = #fields_doc]
                #fields_vis #fields_ident_1: ::rorm::internal::field::FieldProxy<#fields_type, Path>,
            )*
        }
        impl<Path: 'static> ::rorm::model::ConstNew for #ident<Path> {
            const NEW: Self = Self {
                #(
                    #fields_ident_2: ::rorm::internal::field::FieldProxy::new(),
                )*
            };
            const REF: &'static Self = &Self::NEW;
        }
    }
}
