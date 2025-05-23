use proc_macro2::Ident;
use quote::format_ident;
use syn::visit_mut::VisitMut;
use syn::{LitStr, Type, Visibility};

use crate::parse::field_type::{
    ParsedFieldType, ParsedFieldTypeNewType, ParsedFieldTypeStruct, ParsedFieldTypeUnit,
};
use crate::parse::model::{ModelFieldAnnotations, ParsedField};
use crate::utils::to_db_name;

pub fn analyze_field_type(parsed: ParsedFieldType) -> darling::Result<AnalyzedFieldType> {
    match parsed {
        ParsedFieldType::Struct(parsed) => {
            analyze_field_type_struct(parsed).map(AnalyzedFieldType::Struct)
        }
        ParsedFieldType::NewType(parsed) => {
            analyze_field_type_new_type(parsed).map(AnalyzedFieldType::NewType)
        }
        ParsedFieldType::Unit(parsed) => {
            analyze_field_type_unit(parsed).map(AnalyzedFieldType::Unit)
        }
    }
}

fn analyze_field_type_struct(
    ParsedFieldTypeStruct { vis, ident, fields }: ParsedFieldTypeStruct,
) -> darling::Result<AnalyzedFieldTypeStruct> {
    let mut errors = darling::Error::accumulator();

    let mut analyzed_fields = Vec::with_capacity(fields.len());
    let struct_ident = &ident; // alias to avoid shadowing in following loop
    for field in fields {
        let ParsedField {
            vis,
            ident,
            mut ty,
            annos:
                ModelFieldAnnotations {
                    auto_create_time,
                    auto_update_time,
                    auto_increment,
                    primary_key,
                    unique,
                    id,
                    on_delete,
                    on_update,
                    rename,
                    //ignore,
                    default,
                    max_length,
                    index,
                },
        } = field;

        // Get column name
        let column =
            rename.unwrap_or_else(|| LitStr::new(&to_db_name(ident.to_string()), ident.span()));
        if column.value().contains("__") {
            errors.push(darling::Error::custom("Column names can't contain a double underscore. If you need to name your field like this, consider using `#[rorm(rename = \"...\")]`.").with_span(&column));
        }
        if column.value().len() > 63 {
            errors.push(darling::Error::custom("Column names can't be larger than 63 bytes. If you need to name your field like this, consider using `#[rorm(rename = \"...\")]`.").with_span(&column));
        }

        // Replace `Self` in the field's type to the struct's identifier
        struct ReplaceSelf<'a>(&'a Ident);
        impl VisitMut for ReplaceSelf<'_> {
            fn visit_ident_mut(&mut self, i: &mut Ident) {
                if i == "Self" {
                    *i = self.0.clone();
                }
            }
        }
        ReplaceSelf(struct_ident).visit_type_mut(&mut ty);

        analyzed_fields.push(AnalyzedFieldTypeStructField {
            vis,
            unit: format_ident!("__{struct_ident}_{ident}",),
            ident,
            column,
            ty,
            annos: (),
        });
    }

    errors.finish_with(AnalyzedFieldTypeStruct {
        vis,
        ident,
        fields: analyzed_fields,
    })
}

fn analyze_field_type_new_type(
    parsed: ParsedFieldTypeNewType,
) -> darling::Result<AnalyzedFieldTypeNewType> {
    match parsed {}
}

fn analyze_field_type_unit(
    ParsedFieldTypeUnit { ident }: ParsedFieldTypeUnit,
) -> darling::Result<AnalyzedFieldTypeUnit> {
    Ok(AnalyzedFieldTypeUnit { ident })
}

pub enum AnalyzedFieldType {
    Struct(AnalyzedFieldTypeStruct),
    NewType(AnalyzedFieldTypeNewType),
    Unit(AnalyzedFieldTypeUnit),
}

pub struct AnalyzedFieldTypeStruct {
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Vec<AnalyzedFieldTypeStructField>,
}

pub enum AnalyzedFieldTypeNewType {}

pub struct AnalyzedFieldTypeUnit {
    pub ident: Ident,
}

pub struct AnalyzedFieldTypeStructField {
    pub vis: Visibility,
    pub ident: Ident,
    pub column: LitStr,
    pub unit: Ident,
    pub ty: Type,
    pub annos: (),
}
