use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::ops::{Deref, DerefMut};

pub struct Annotation {
    pub span: Span,
    pub field: &'static str,
    pub variant: &'static str,
    pub expr: Option<TokenStream>,
}

pub struct Annotations(Vec<Annotation>);
impl Annotations {
    pub const fn new() -> Self {
        Annotations(Vec::new())
    }

    pub fn iter_steps<'a>(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.iter().map(|anno| {
            let field = Ident::new(anno.field, anno.span.clone());
            let variant = Ident::new(anno.variant, anno.span.clone());
            if let Some(expr) = anno.expr.as_ref() {
                quote! {
                    .#field(::rorm::hmr::annotations::#variant(#expr))
                }
            } else {
                quote! {
                    .#field(::rorm::hmr::annotations::#variant)
                }
            }
        })
    }

    pub fn get_type(&self, value_type: &syn::Type) -> TokenStream {
        let mut anno_type = quote! {
            <#value_type as ::rorm::model::AsDbType>::Annotations
        };
        for anno in self.iter() {
            let anno = Ident::new(anno.variant, anno.span.clone());
            anno_type = quote! {
                ::rorm::annotation_builder::Add<::rorm::hmr::annotations::#anno, #anno_type>
            };
        }
        anno_type
    }
}
impl Deref for Annotations {
    type Target = Vec<Annotation>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Annotations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
