use proc_macro2::Ident;
use quote::format_ident;
use syn::{LitInt, LitStr, Type, Visibility};

use crate::analyze::vis_to_display;
use crate::parse::annotations::{Default, Index, OnAction};
use crate::parse::model::{ModelAnnotations, ModelFieldAnnotations, ParsedField, ParsedModel};
use crate::utils::to_db_name;

pub fn analyze_model(parsed: ParsedModel) -> darling::Result<AnalyzedModel> {
    let ParsedModel {
        vis,
        ident,
        annos: ModelAnnotations { rename },
        fields,
    } = parsed;
    let mut errors = darling::Error::accumulator();

    // Get table name
    let table = rename.unwrap_or_else(|| LitStr::new(&to_db_name(ident.to_string()), ident.span()));
    if table.value().contains("__") {
        errors.push(darling::Error::custom("Table names can't contain a double underscore. If you need to name your model like this, consider using `#[rorm(rename = \"...\")]`.").with_span(&table));
    }

    // Analyze fields
    let mut ignored = Vec::new();
    let mut analyzed_fields = Vec::with_capacity(
        /* assuming most fields won't be ignored */
        fields.len(),
    );
    let model_ident = &ident; // alias to avoid shadowing in following loop
    for field in fields {
        let ParsedField {
            vis,
            ident,
            ty,
            annos:
                ModelFieldAnnotations {
                    auto_create_time,
                    auto_update_time,
                    mut auto_increment,
                    mut primary_key,
                    unique,
                    id,
                    on_delete,
                    on_update,
                    rename,
                    ignore,
                    default,
                    max_length,
                    index,
                },
        } = field;
        if ignore {
            ignored.push(ident);
        } else {
            // Get column name
            let column =
                rename.unwrap_or_else(|| LitStr::new(&to_db_name(ident.to_string()), ident.span()));
            if column.value().contains("__") {
                errors.push(darling::Error::custom("Column names can't contain a double underscore. If you need to name your field like this, consider using `#[rorm(rename = \"...\")]`.").with_span(&column));
            }

            // Handle #[rorm(id)] annotation
            if id {
                if primary_key {
                    errors.push(
                        darling::Error::custom(
                            "`#[rorm(primary_key)]` is implied by `#[rorm(id)]`. Please remove one of them.",
                        )
                        .with_span(&ident),
                    );
                }
                if auto_increment {
                    errors.push(
                        darling::Error::custom(
                            "`#[rorm(auto_increment)]` is implied by `#[rorm(id)]`. Please remove one of them.",
                        )
                        .with_span(&ident),
                    );
                }
                primary_key = true;
                auto_increment = true;
            }

            analyzed_fields.push(AnalyzedField {
                vis,
                unit: format_ident!("__{}_{}", model_ident, ident),
                ident,
                column,
                ty,
                annos: AnalyzedModelFieldAnnotations {
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
                },
            });
        }
    }

    // Find the unique primary key
    let mut primary_keys = Vec::with_capacity(1); // Should be exactly one
    for (index, field) in analyzed_fields.iter().enumerate() {
        if field.annos.primary_key {
            primary_keys.push((index, field));
        }
    }
    let mut primary_key = usize::MAX; // will only be returned if it is set properly
    match primary_keys.as_slice() {
        [(index, _)] => primary_key = *index,
        [] => errors.push(
            darling::Error::custom(format!(
                "Model misses a primary key. Try adding the default one:\n\n#[rorm(id)]\n{vis}id: i64,", vis = vis_to_display(&vis),
            ))
            .with_span(&ident),
        ),
        _ => errors.push(darling::Error::multiple(
            primary_keys
                .into_iter()
                .map(|(_, field)| {
                    darling::Error::custom("Model has more than one primary key. Please remove all but one of them.")
                        .with_span(&field.ident)
                })
                .collect(),
        )),
    }

    errors.finish_with(AnalyzedModel {
        vis,
        ident,
        table,
        ignored,
        fields: analyzed_fields,
        primary_key,
    })
}

pub struct AnalyzedModel {
    pub vis: Visibility,
    pub ident: Ident,
    pub table: LitStr,
    pub ignored: Vec<Ident>,
    pub fields: Vec<AnalyzedField>,
    /// the primary key's index
    pub primary_key: usize,
}

pub struct AnalyzedField {
    pub vis: Visibility,
    pub ident: Ident,
    pub column: LitStr,
    pub unit: Ident,
    pub ty: Type,
    pub annos: AnalyzedModelFieldAnnotations,
}

pub struct AnalyzedModelFieldAnnotations {
    pub auto_create_time: bool,
    pub auto_update_time: bool,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub on_delete: Option<OnAction>,
    pub on_update: Option<OnAction>,
    pub default: Option<Default>,
    pub max_length: Option<LitInt>,
    pub index: Option<Index>,
}
