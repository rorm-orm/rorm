use quote::ToTokens;
use syn::{VisRestricted, Visibility};

pub mod field_type;
pub mod model;

pub fn vis_to_display(vis: &Visibility) -> impl std::fmt::Display + '_ {
    DisplayableVisibility(vis)
}
struct DisplayableVisibility<'a>(&'a Visibility);
impl std::fmt::Display for DisplayableVisibility<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Visibility::Public(_) => f.write_str("pub "),
            Visibility::Restricted(VisRestricted {
                pub_token: _,
                paren_token: _,
                in_token,
                path,
            }) => {
                write!(
                    f, "pub({in}{path}) ",
                    in = if in_token.is_some() { "in " } else { "" },
                    path = path.to_token_stream())
            }
            Visibility::Inherited => Ok(()),
        }
    }
}
