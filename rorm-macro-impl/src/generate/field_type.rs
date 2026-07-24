#![allow(
    non_snake_case,
    reason = "We use `__` in variables used for quote to indicate a field access (ex `let foo__bar = &foo.bar`)"
)]

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::LitStr;

use crate::analyze::field_type::{AnalyzedField, AnalyzedFieldAnnotations, AnalyzedFieldType};
use crate::generate::utils::get_source;
use crate::generate::SliceExt;
use crate::parse::annotations::{Index, NamedIndex, OnAction};
use crate::MacroConfig;

pub fn generate_field_type(field_type: &AnalyzedFieldType, config: &MacroConfig) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

    let AnalyzedFieldType {
        vis,
        ident,
        fields,
        get_names,
        get_annotations,
        check,
        decoder,
    } = field_type;

    let fields_struct_ident = format_ident!("__{}_Fields_Struct", field_type.ident);
    let field_declarations = generate_fields(field_type, config);

    let fields__vis = field_type.fields.map_collect(|x| &x.vis);
    let fields__ident = fields.map_collect(|x| &x.ident);
    let fields__unit: Vec<_> = fields.map_collect(|x| &x.unit);
    let fields__column = fields.map_collect(|x| &x.column);
    let fields__get_name = fields.map_collect(|x| &x.get_name);
    let fields__ty: Vec<_> = fields.map_collect(|x| &x.ty);

    quote! {const _: () = {
        #field_declarations

        const NUM_COLUMNS: usize = {
            0 #(+ <<#fields__ty as #rorm_path::fields::traits::FieldType>::Columns as #rorm_path::fields::traits::Columns>::NUM)*
        };
        impl #rorm_path::fields::traits::FieldType for #ident {
            type Columns = #rorm_path::fields::traits::Array<NUM_COLUMNS>;

            const NULL: #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::db::sql::value::NullType> = {
                let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                    [#rorm_path::db::sql::value::NullType::Bool; NUM_COLUMNS]
                );
                #(
                    builder.extend_const(
                        <#fields__ty as #rorm_path::fields::traits::FieldType>::NULL
                    );
                )*
                builder.finish_const()
            };

            fn into_values<'a>(self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'a>> {
                let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                    ::std::array::from_fn(|_| #rorm_path::conditions::Value::Bool(false))
                );
                #(
                    self.#fields__ident.into_values();
                )*
                builder.finish()
            }

            fn as_values(&self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'_>> {
                let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                    ::std::array::from_fn(|_| #rorm_path::conditions::Value::Bool(false))
                );
                #(
                    self.#fields__ident.as_values();
                )*
                builder.finish()
            }

            type GetNames = #get_names;
            type GetAnnotations = #get_annotations;
            type Check = #check;
            type Decoder = #decoder;
        }

        impl<__Field, __Path> #rorm_path::internal::field::ContainerFieldType<__Field, __Path> for #ident
        where
            Self: #rorm_path::fields::traits::FieldType,
            __Field: #rorm_path::internal::field::Field<Type = Self>,
            __Path: #rorm_path::internal::relation_path::Path<Current = __Field::Model>,
        {
            type Target = #fields_struct_ident<__Field, __Path>;
        }

        #[doc = #rorm_path::doc_concat!("Subfield of [`" #ident "`]")]
        #[allow(non_camel_case_types)]
        #vis struct #fields_struct_ident <
            __Field: #rorm_path::internal::field::Field<Type = #ident>,
            __Path: #rorm_path::internal::relation_path::Path,
        > {
            #(
                #[doc = #rorm_path::doc_concat!("[`" #ident "`]'s `" #fields__ident "` field")]
                #fields__vis #fields__ident: #rorm_path::fields::proxy::FieldProxy<(#fields__unit <__Field>, __Path)>,
            )*
        }
        impl <
            __Field: #rorm_path::internal::field::Field<Type = #ident>,
            __Path: #rorm_path::internal::relation_path::Path
        > #rorm_path::internal::ConstRef for #fields_struct_ident <__Field, __Path> {
            const REF: &'static Self = &Self {
                #(
                    #fields__ident: #rorm_path::fields::proxy::new(),
                )*
            };
        }

        #rorm_path::const_fn! {
            #vis fn #get_names(
                #[raw] T: (#rorm_path::fields::utils::column_name::ColumnName,)
            ) -> #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::fields::utils::column_name::ColumnName> {
                let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                    [#rorm_path::fields::utils::column_name::ColumnName::placeholder(); NUM_COLUMNS]
                );
                #(
                    builder.extend_const(
                        <<<#fields__ty as #rorm_path::fields::traits::FieldType>::GetNames as #rorm_path::fields::utils::const_fn::ConstFn<_, _>>::Body<(
                            <#fields__get_name as #rorm_path::fields::utils::const_fn::ConstFn<_, _>>::Body<T>,
                        )> as #rorm_path::fields::utils::const_fn::Contains<_>>::ITEM,
                    );
                )*
                builder.finish_const()
            }
        }

        #(
            #rorm_path::const_fn! {
                #[allow(non_snake_case)]
                #vis fn #fields__get_name(
                    column_name: #rorm_path::fields::utils::column_name::ColumnName
                ) -> #rorm_path::fields::utils::column_name::ColumnName {
                    column_name.join(
                        #fields__column
                    )
                }
            }
        )*

        #rorm_path::const_fn! {
            #vis fn #get_annotations(
                #[raw] T: (#rorm_path::internal::hmr::annotations::Annotations,)
            ) -> #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations> {
                let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                    [#rorm_path::internal::hmr::annotations::Annotations::empty(); NUM_COLUMNS]
                );
                #(
                    builder.extend_const(
                        <<<#fields__ty as #rorm_path::fields::traits::FieldType>::GetAnnotations as #rorm_path::fields::utils::const_fn::ConstFn<_, _>>::Body<T> as #rorm_path::fields::utils::const_fn::Contains<_>>::ITEM,
                    );
                )*
                builder.finish_const()
            }
        }

        #rorm_path::const_fn! {
            #vis fn #check(
                #[raw] T: (#rorm_path::internal::hmr::annotations::Annotations, #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>)
            ) -> Result<(), #rorm_path::internal::const_concat::ConstString<1024>> {
                // TODO: how?!
                Ok(())
            }
        }

        #vis struct #decoder {#(
            pub #fields__ident: <#fields__ty as #rorm_path::fields::traits::FieldType>::Decoder,
        )*}

        impl #rorm_path::crud::decoder::Decoder for #decoder {
            type Result = #ident;

            fn by_name<'index>(&'index self, row: &'_ #rorm_path::db::Row) -> Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                Ok(#ident {#(
                    #fields__ident: self.#fields__ident.by_name(row)?,
                )*})
            }

            fn by_index<'index>(&'index self, row: &'_ #rorm_path::db::Row) -> Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                Ok(#ident {#(
                    #fields__ident: self.#fields__ident.by_index(row)?,
                )*})
            }
        }

        impl #rorm_path::internal::field::decoder::FieldDecoder for #decoder {
            fn new<I>(ctx: &mut #rorm_path::internal::query_context::QueryContext, _: #rorm_path::fields::proxy::FieldProxy<I>) -> Self
            where
                I: #rorm_path::fields::proxy::FieldProxyImpl<Field: #rorm_path::internal::field::Field<Type= Self::Result>>,
            {
                Self {#(
                    #fields__ident: <<#fields__ty as #rorm_path::fields::traits::FieldType>::Decoder as #rorm_path::internal::field::decoder::FieldDecoder>::new(
                        &mut *ctx, #rorm_path::fields::proxy::new::<(#fields__unit<I::Field>, I::Path)>()
                    ),
                )*}
            }
        }
    };}
}

fn generate_fields(model: &AnalyzedFieldType, config: &MacroConfig) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

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

            get_name: _,
        } = field;

        let source = get_source(ident.span(), config);
        let vis = &model.vis;
        let doc = LitStr::new(
            &format!("rorm's representation of [`{model_ident}`]'s `{ident}` field",),
            ident.span(),
        );
        let annos = generate_field_annotations(annos, config);

        tokens.extend(quote_spanned! {ident.span()=>
            #[doc = #doc]
            #[allow(non_camel_case_types)]
            #vis struct #unit <__Field: #rorm_path::internal::field::Field<Type = #model_ident>> ( std::marker::PhantomData<__Field> );
            impl <__Field: #rorm_path::internal::field::Field<Type = #model_ident>> ::std::clone::Clone for #unit <__Field> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl <__Field: #rorm_path::internal::field::Field<Type = #model_ident>> ::std::marker::Copy for #unit <__Field> {}
            impl <__Field: #rorm_path::internal::field::Field<Type = #model_ident>> #rorm_path::internal::field::Field for #unit <__Field> {
                type Type = #ty;
                type Model = <__Field as #rorm_path::internal::field::Field>::Model;
                const INDEX: usize = #index;
                const NAME: #rorm_path::fields::utils::column_name::ColumnName = #rorm_path::fields::utils::column_name::ColumnName::new(#column);
                const EXPLICIT_ANNOTATIONS: #rorm_path::internal::hmr::annotations::Annotations = #annos;
                const SOURCE: #rorm_path::internal::hmr::Source = #source;
                fn new() -> Self {
                    Self(::std::marker::PhantomData)
                }
            }
        });
        // TODO: run checker?
    }
    tokens
}

fn generate_field_annotations(
    annos: &AnalyzedFieldAnnotations,
    config: &MacroConfig,
) -> TokenStream {
    let MacroConfig {
        rorm_path,
        non_exhaustive: _,
    } = config;

    let AnalyzedFieldAnnotations {
        unique,
        on_delete,
        on_update,
        default,
        max_length,
        index,
    } = annos;

    // Convert every field into its "creation" expression
    let unique = unique.then(|| quote! {Unique});
    let max_length = max_length.as_ref().map(|len| quote! {MaxLength(#len)});
    let default = default.as_ref().map(|default| {
        let variant = Ident::new(default.variant, default.literal.span());
        let literal = &default.literal;
        quote! {DefaultValue(#rorm_path::internal::hmr::annotations::DefaultValueData::#variant(#literal))}
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
                quote! { Index(Some(#rorm_path::internal::hmr::annotations::IndexData { name: #name, priority: None })) }
            }

            Some(NamedIndex {
                     name,
                     priority: Some(priority),
                 }) => {
                quote! { Index(Some(#rorm_path::internal::hmr::annotations::IndexData { name: #name, priority: Some(#priority) })) }
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
            quote! {Some(#rorm_path::internal::hmr::annotations::#token)}
        } else {
            quote! {None}
        }
    };
    let default = finalize(default);
    let index = finalize(index);
    let max_length = finalize(max_length);
    let on_delete = finalize(on_delete);
    let on_update = finalize(on_update);
    let unique = finalize(unique);

    quote! {
        #rorm_path::internal::hmr::annotations::Annotations {
            default: #default,
            index: #index,
            max_length: #max_length,
            on_delete: #on_delete,
            on_update: #on_update,
            unique: #unique,
            ..#rorm_path::internal::hmr::annotations::Annotations::empty()
        }
    }
}
