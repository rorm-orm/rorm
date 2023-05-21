use darling::{Error, FromAttributes, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::trait_impls;

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
    let mut field_types = Vec::new();
    for field in strct.fields {
        errors.handle(NoAnnotations::from_attributes(&field.attrs));
        if let Some(ident) = field.ident {
            field_idents.push(ident);
            field_types.push(field.ty);
        } else {
            errors.push(Error::custom("missing field name").with_span(&field));
        }
    }

    let Some(PatchAnnotations {model: ModelPath(model_path)}) = errors.handle(PatchAnnotations::from_attributes(&strct.attrs)) else {
        return errors.finish_with(TokenStream::new());
    };

    let patch = strct.ident;
    let compile_check = format_ident!("__compile_check_{}", patch);
    let impl_patch =
        trait_impls::patch(&strct.vis, &patch, &model_path, &field_idents, &field_types);

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
    })
}
