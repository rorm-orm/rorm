use darling::ast::NestedMeta;
use darling::{Error, FromAttributes, FromMeta};
use proc_macro2::Ident;
use syn::{Lit, LitInt, LitStr, Meta};

#[derive(FromAttributes, Debug)]
#[darling(attributes(rorm))]
pub struct NoAnnotations;

#[derive(Debug)]
pub struct Default {
    pub variant: &'static str,
    pub literal: Lit,
}
impl FromMeta for Default {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(Default {
            variant: match value {
                Lit::Str(_) => Ok("String"),
                Lit::Int(_) => Ok("Integer"),
                Lit::Float(_) => Ok("Float"),
                Lit::Bool(_) => Ok("Boolean"),
                _ => Err(Error::unexpected_lit_type(value)),
            }?,
            literal: value.clone(),
        })
    }
}

#[derive(Debug)]
pub struct OnAction(pub Ident);
impl FromMeta for OnAction {
    fn from_value(lit: &Lit) -> darling::Result<Self> {
        static OPTIONS: [&str; 4] = ["Restrict", "Cascade", "SetNull", "SetDefault"];
        (match lit {
            Lit::Str(string) => {
                let string = string.value();
                let value = string.as_str();
                if OPTIONS.contains(&value) {
                    Ok(OnAction(Ident::new(value, lit.span())))
                } else {
                    Err(Error::unknown_field_with_alts(value, &OPTIONS))
                }
            }
            _ => Err(Error::unexpected_lit_type(lit)),
        })
        .map_err(|e| e.with_span(lit))
    }
}

#[derive(Default, Debug)]
pub struct Index(pub Option<NamedIndex>);
impl FromMeta for Index {
    fn from_word() -> darling::Result<Self> {
        Ok(Index(None))
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            Ok(Index(None))
        } else {
            Ok(Index(Some(NamedIndex::from_list(items)?)))
        }
    }
}

#[derive(FromMeta, Debug)]
pub struct NamedIndex {
    pub name: LitStr,
    pub priority: Option<LitInt>,
}

/// The fields a model's `#[rorm(unique(...))]` spans
#[derive(Debug)]
pub struct Unique(pub Vec<Ident>);
impl FromMeta for Unique {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            return Err(Error::too_few_items(1));
        }

        let mut errors = Error::accumulator();
        let fields = items
            .iter()
            .filter_map(|item| {
                errors.handle(match item {
                    NestedMeta::Meta(Meta::Path(path)) => {
                        path.get_ident().cloned().ok_or_else(|| {
                            Error::custom("Expected a field's identifier").with_span(path)
                        })
                    }
                    _ => Err(Error::custom("Expected a field's identifier").with_span(item)),
                })
            })
            .collect();
        errors.finish_with(Unique(fields))
    }
}
