use std::path::Path;
use std::{env, fmt, fs};

use datatest_stable::{harness, Result, Utf8Path};
use proc_macro2::{Ident, TokenStream};
use syn::__private::ToTokens;

harness! {
    { test = compile, root = "tests/data/derives/", pattern = "^[^/]+$" },
    { test = expand, root = "tests/data/derives/", pattern = "^[^/]+$" },
}

fn compile(path: &Path) -> Result<()> {
    trybuild::TestCases::new().pass(path);
    Ok(())
}

fn expand(input_file: &Utf8Path, input_str: String) -> Result<()> {
    let expansions_dir =
        input_file.with_file_name(format!("{}_expansions", input_file.file_stem().unwrap()));

    let input_ast = syn::parse_file(&input_str).context("Failed to parse input")?;

    for item in &input_ast.items {
        let Some((ident, derive_fn)) = get_derive_fn(item)? else {
            continue;
        };

        let expansion = derive_fn(item.to_token_stream());

        let expansion_str =
            prettyplease::unparse(&syn::parse2(expansion).context("Failed to parse expansion")?);

        let expansion_path = expansions_dir.join(format!("{ident}.rs"));

        if !expansion_path.exists()
            || &fs::read_to_string(&expansion_path).context("Failed to read expansion from file")?
                != &expansion_str
        {
            if env::var_os("RORM_WRITE_EXPANSION").is_some() {
                fs::create_dir_all(&expansions_dir)
                    .context("Failed to create directory for expansions")?;
                fs::write(&expansion_path, &expansion_str)
                    .context("Failed to write expansion to file")?;
            } else {
                return Err(format!("Expansion of {ident} doesn't match").into());
            }
        }
    }

    Ok(())
}

fn get_derive_fn(item: &syn::Item) -> Result<Option<(Ident, fn(TokenStream) -> TokenStream)>> {
    fn is_derive(attr: &&syn::Attribute) -> bool {
        matches!(attr.path().get_ident(), Some(ident) if ident == "derive")
    }

    let Some((derive_attr, item_ident)) = (match &item {
        syn::Item::Struct(item) => item.attrs.iter().find(is_derive).zip(Some(&item.ident)),
        syn::Item::Enum(item) => item.attrs.iter().find(is_derive).zip(Some(&item.ident)),
        _ => None,
    }) else {
        return Ok(None);
    };

    let derive_paths = derive_attr
        .parse_args_with(|parser: syn::parse::ParseStream| {
            let mut paths = Vec::new();
            while !parser.is_empty() {
                let path: syn::Path = parser.parse()?;
                paths.push(path);

                if parser.peek(syn::Token![,]) {
                    let _: syn::Token![,] = parser.parse()?;
                } else {
                    break;
                }
            }
            Ok(paths)
        })
        .context("Derive attribute is malformed")?;

    for path in derive_paths {
        let Some(segment) = path.segments.last() else {
            continue;
        };
        let ident = &segment.ident;

        return Ok(Some((
            item_ident.clone(),
            if ident == "Model" {
                |input: TokenStream| rorm_macro_impl::derive_model(input, Default::default())
            } else if ident == "Patch" {
                |input: TokenStream| rorm_macro_impl::derive_patch(input, Default::default())
            } else if ident == "DbEnum" {
                |input: TokenStream| rorm_macro_impl::derive_db_enum(input, Default::default())
            } else if ident == "FieldType" {
                |input: TokenStream| rorm_macro_impl::derive_field_type(input, Default::default())
            } else {
                continue;
            },
        )));
    }

    Ok(None)
}

trait ResultExt {
    type Ok;
    fn context(self, context: &'static str) -> Result<Self::Ok>;
}
impl<T, E: fmt::Display> ResultExt for std::result::Result<T, E> {
    type Ok = T;
    fn context(self, context: &'static str) -> Result<T> {
        Ok(self.map_err(|e| format!("{context}: {e}"))?)
    }
}
