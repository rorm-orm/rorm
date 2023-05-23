use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Visibility;

pub mod model;

pub fn vis_to_display(vis: &Visibility) -> impl std::fmt::Display + '_ {
    DisplayableVisibility(vis)
}
struct DisplayableVisibility<'a>(&'a Visibility);
impl<'a> std::fmt::Display for DisplayableVisibility<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Visibility::Public(_) => f.write_str("pub "),
            Visibility::Crate(_) => f.write_str("pub(crate) "),
            Visibility::Restricted(data) => {
                let mut path = TokenStream::new();
                data.path.to_tokens(&mut path);
                write!(f, "pub(in {path}) ")
            }
            Visibility::Inherited => Ok(()),
        }
    }
}
