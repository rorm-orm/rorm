use proc_macro2::Ident;
use quote::format_ident;
use syn::visit_mut::VisitMut;
use syn::{LitInt, LitStr, Type, Visibility};

use crate::parse::annotations::{Index, OnAction};
use crate::parse::field_type::{
    FieldTypeAnnotations, FieldTypeFieldAnnotations, ParsedField, ParsedFieldType,
};
use crate::utils::to_db_name;

pub fn analyze_field_type(parsed: ParsedFieldType) -> darling::Result<AnalyzedFieldType> {
    let ParsedFieldType {
        vis,
        ident,
        annos: FieldTypeAnnotations {},
        fields,
    } = parsed;
    let mut errors = darling::Error::accumulator();

    let mut analyzed_fields = Vec::new();
    let model_ident = &ident; // alias to avoid shadowing in following loop
    for field in fields {
        let ParsedField {
            vis,
            ident,
            mut ty,
            annos:
                FieldTypeFieldAnnotations {
                    unique,
                    on_delete,
                    on_update,
                    rename,
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

        // Replace `Self` in the field's type to the model's identifier
        struct ReplaceSelf<'a>(&'a Ident);
        impl VisitMut for ReplaceSelf<'_> {
            fn visit_ident_mut(&mut self, i: &mut Ident) {
                if i == "Self" {
                    *i = self.0.clone();
                }
            }
        }
        ReplaceSelf(model_ident).visit_type_mut(&mut ty);

        analyzed_fields.push(AnalyzedField {
            vis,
            ident: ident.clone(),
            column,
            ty,
            annos: AnalyzedFieldAnnotations {
                unique,
                on_delete,
                on_update,
                default,
                max_length,
                index,
            },
            unit: format_ident!("__{}_{}", model_ident, ident),
            get_name: format_ident!("get_{model_ident}_{ident}_name"),
        });
    }

    errors.finish_with(AnalyzedFieldType {
        vis,
        ident: ident.clone(),
        fields: analyzed_fields,
        get_names: format_ident!("get_{ident}_names"),
        get_annotations: format_ident!("get_{ident}_annotations"),
        check: format_ident!("check_{ident}"),
        decoder: format_ident!("{ident}Decoder"),
    })
}

pub struct AnalyzedFieldType {
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Vec<AnalyzedField>,

    // Precomputed identifier
    pub get_names: Ident,
    pub get_annotations: Ident,
    pub check: Ident,
    pub decoder: Ident,
}

pub struct AnalyzedField {
    pub vis: Visibility,
    pub ident: Ident,
    pub column: LitStr,
    pub ty: Type,
    pub annos: AnalyzedFieldAnnotations,

    // Precomputed identifier
    pub unit: Ident,
    pub get_name: Ident,
}

pub struct AnalyzedFieldAnnotations {
    pub unique: bool,
    pub on_delete: Option<OnAction>,
    pub on_update: Option<OnAction>,
    pub default: Option<crate::parse::annotations::Default>,
    pub max_length: Option<LitInt>,
    pub index: Option<Index>,
}
