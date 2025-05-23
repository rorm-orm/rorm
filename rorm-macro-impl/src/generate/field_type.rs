use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::analyze::field_type::{
    AnalyzedFieldType, AnalyzedFieldTypeNewType, AnalyzedFieldTypeStruct, AnalyzedFieldTypeUnit,
};
use crate::MacroConfig;

pub fn generate_field_type(analyzed: &AnalyzedFieldType, config: &MacroConfig) -> TokenStream {
    match analyzed {
        AnalyzedFieldType::Struct(analyzed) => generate_field_type_struct(analyzed, config),
        AnalyzedFieldType::NewType(analyzed) => generate_field_type_new_type(analyzed, config),
        AnalyzedFieldType::Unit(analyzed) => generate_field_type_unit(analyzed, config),
    }
}

fn generate_field_type_struct(
    AnalyzedFieldTypeStruct { vis, ident, fields }: &AnalyzedFieldTypeStruct,
    MacroConfig {
        rorm_path,
        non_exhaustive: _,
    }: &MacroConfig,
) -> TokenStream {
    let fields_len = fields.len();
    let fields_type = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let fields_ident = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();

    let decoder = format_ident!("__{ident}_Decoder");
    let get_names = format_ident!("__{ident}_GetNames");
    let get_annotations = format_ident!("__{ident}_GetAnnotations");
    let check = format_ident!("__{ident}_Check");
    let get_names_body = format_ident!("__{ident}_GetNames_Body");
    let get_annotations_body = format_ident!("__{ident}_GetAnnotations_Body");
    let check_body = format_ident!("__{ident}_Check_Body");

    quote! {
        const _: () = {
            impl #rorm_path::fields::traits::FieldType for #ident {
                type Columns = #rorm_path::fields::traits::Array<#fields_len>;

                const NULL: #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::db::sql::value::NullType> = {
                    let mut builder = #rorm_path::internal::field::mulit_column::ArrayBuilder::new(
                        [NullType::Bool; #fields_len]
                    );
                    #(
                        builder.extend_const(<#fields_type as #rorm_path::fields::traits::FieldType>::NULL);
                    )*
                    builder.finish_const()
                };

                fn into_values<'a>(self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'a>> {
                    let mut builder = #rorm_path::internal::field::mulit_column::ArrayBuilder::new(
                        [#rorm_path::conditions::Value::Bool(false); #fields_len]
                    );
                    #(
                        builder.extend(#rorm_path::fields::traits::FieldType::into_values(self.#fields_ident));
                    )*
                    builder.finish()
                }

                fn as_values(&self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'_>> {
                    let mut builder = #rorm_path::internal::field::mulit_column::ArrayBuilder::new(
                        [#rorm_path::conditions::Value::Bool(false); #fields_len]
                    );
                    #(
                        builder.extend(#rorm_path::fields::traits::FieldType::into_values(self.#fields_ident));
                    )*
                    builder.finish()
                }

                type Decoder = #decoder;
                type GetNames = #get_names;
                type GetAnnotations = #get_annotations;
                type Check = #check;
            }

            #[allow(non_camel_case_types)]
            #vis struct #decoder {#(
                #fields_ident: <#fields_type as #rorm_path::fields::traits::FieldType>::Decoder,
            )*}
            impl #rorm_path::crud::decoder::Decoder for #decoder {
                type Result = #ident;

                fn by_name<'index>(&'index self, row: &'_ #rorm_path::Row) -> ::std::result::Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                    Ok(#ident {#(
                        #fields_ident: #rorm_path::crud::decoder::Decoder::by_name(&self.#fields_ident, row)?,
                    )*})
                }

                fn by_index<'index>(&'index self, row: &'_ #rorm_path::Row) -> ::std::result::Result<Self::Result, #rorm_path::db::row::RowError<'index>> {
                    Ok(#ident {#(
                        #fields_ident: #rorm_path::crud::decoder::Decoder::by_index(&self.#fields_ident, row)?,
                    )*})
                }
            }
            impl #rorm_path::internal::field::decoder::FieldDecoder for #decoder {
                fn new<I>(
                    ctx: &mut #rorm_path::internal::query_context::QueryContext,
                    _: #rorm_path::fields::proxy::FieldProxy<I>
                ) -> Self
                where
                    I: #rorm_path::fields::proxy::FieldProxyImpl<
                        Field: #rorm_path::internal::field::Field<Type = Self::Result>
                    >,
                {
                    todo!()
                }
            }

            #[allow(non_camel_case_types)]
            #vis struct #get_names;
            impl #rorm_path::fields::utils::const_fn::ConstFn<
                (#rorm_path::fields::utils::column_name::ColumnName,),
                #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::fields::utils::column_name::ColumnName>,
            > for #get_names {
                type Body<T: Contains<(#rorm_path::fields::utils::column_name::ColumnName,)>> = #get_names_body<T>;
            }
            #vis struct #get_names_body<T>(::std::marker::PhantomData<T>);
            impl<T> Contains<#rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::fields::utils::column_name::ColumnName>>
            for #get_names_body<T>
            where
                T: Contains<(#rorm_path::fields::utils::column_name::ColumnName,)>
            {
                const ITEM: #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::fields::utils::column_name::ColumnName> = todo!();
            }

            #[allow(non_camel_case_types)]
            #vis struct #get_annotations;
            impl #rorm_path::fields::utils::const_fn::ConstFn<
                (#rorm_path::internal::hmr::annotations::Annotations,),
                #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>,
            > for #get_annotations {
                type Body<T: Contains<(#rorm_path::internal::hmr::annotations::Annotations,)>> = #get_annotations_body<T>;
            }
            #vis struct #get_annotations_body<T>(::std::marker::PhantomData<T>);
            impl<T> Contains<#rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>>
            for #get_annotations_body<T>
            where
                T: Contains<(#rorm_path::internal::hmr::annotations::Annotations,)>
            {
                const ITEM: #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations> = todo!();
            }

            #[allow(non_camel_case_types)]
            #vis struct #check;
            impl #rorm_path::fields::utils::const_fn::ConstFn<
                (#rorm_path::internal::hmr::annotations::Annotations, #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>),
                ::std::result::Result<(), ConstString<1024>>,
            > for #check {
                type Body<T: Contains<(#rorm_path::internal::hmr::annotations::Annotations, #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>)>> = #check_body<T>;
            }
            #vis struct #check_body<T>(::std::marker::PhantomData<T>);
            impl<T> Contains<::std::result::Result<(), ConstString<1024>>>
            for #check_body<T>
            where
                T: Contains<(#rorm_path::internal::hmr::annotations::Annotations, #rorm_path::fields::traits::FieldColumns<#ident, #rorm_path::internal::hmr::annotations::Annotations>)>
            {
                const ITEM: ::std::result::Result<(), ConstString<1024>> = todo!();
            }
        };
    }
}

fn generate_field_type_new_type(
    analyzed: &AnalyzedFieldTypeNewType,
    MacroConfig {
        rorm_path,
        non_exhaustive: _,
    }: &MacroConfig,
) -> TokenStream {
    match *analyzed {}
}

fn generate_field_type_unit(
    AnalyzedFieldTypeUnit { ident }: &AnalyzedFieldTypeUnit,
    MacroConfig {
        rorm_path,
        non_exhaustive: _,
    }: &MacroConfig,
) -> TokenStream {
    quote! {
        impl #rorm_path::fields::traits::FieldType for #ident {
            type Columns = #rorm_path::fields::traits::Array<0>;

            const NULL: #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::db::sql::value::NullType> = [];

            fn into_values<'a>(self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'a>> {
                []
            }

            fn as_values(&self) -> #rorm_path::fields::traits::FieldColumns<Self, #rorm_path::conditions::Value<'_>> {
                []
            }

            type Decoder = #rorm_path::crud::decoder::NoopDecoder<Self>;

            type GetNames = #rorm_path::fields::utils::get_names::no_columns_names;

            type GetAnnotations = #rorm_path::fields::utils::get_annotations::forward_annotations::<0>;

            type Check = #rorm_path::fields::utils::check::disallow_annotations_check::<0>;
        }
    }
}
