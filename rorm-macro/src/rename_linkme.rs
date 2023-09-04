//! Walk through an ast replacing all `::linkme` with `::rorm::linkme`

use syn::spanned::Spanned;

pub fn rename_expr(expr: &mut syn::Expr) {
    use syn::Expr::*;
    match expr {
        Block(syn::ExprBlock {
            block: syn::Block { stmts, .. },
            ..
        }) => {
            for stmt in stmts.iter_mut() {
                rename_stmt(stmt);
            }
        }
        Reference(syn::ExprReference { expr, .. }) => rename_expr(expr),
        Path(syn::ExprPath { qself, path, .. }) => {
            if let Some(qself) = qself.as_mut() {
                rename_type(&mut qself.ty);
            }
            rename_path(path);
        }
        Call(syn::ExprCall { func, args, .. }) => {
            rename_expr(func);
            for arg in args.iter_mut() {
                rename_expr(arg);
            }
        }
        _ => todo!("Missing syn::Expr variant"),
    }
}

pub fn rename_stmt(stmt: &mut syn::Stmt) {
    use syn::Stmt::*;
    match stmt {
        Item(syn::Item::Fn(syn::ItemFn { sig, block, .. })) => {
            if let syn::ReturnType::Type(_, ty) = &mut sig.output {
                rename_type(ty);
            }
            for arg in sig.inputs.iter_mut() {
                if let syn::FnArg::Typed(pt) = arg {
                    rename_type(&mut pt.ty);
                }
            }
            for stmt in block.stmts.iter_mut() {
                rename_stmt(stmt);
            }
        }
        Local(syn::Local { init, .. }) => {
            if let Some((_, expr)) = init.as_mut() {
                rename_expr(expr);
            }
        }
        Expr(expr) => rename_expr(expr),
        _ => todo!("Missing syn::Stmt variant"),
    }
}

pub fn rename_type(ty: &mut syn::Type) {
    use syn::Type::*;
    match ty {
        Path(syn::TypePath { qself, path }) => {
            if let Some(qself) = qself.as_mut() {
                rename_type(&mut qself.ty);
            }
            rename_path(path);
        }
        _ => todo!("Missing syn::Type variant"),
    }
}

pub fn rename_path(path: &mut syn::Path) {
    // Check if path is absolute
    if path.leading_colon.is_none() {
        return;
    }

    // Check if path points to linkme
    if path
        .segments
        .first()
        .map_or(true, |segment| segment.ident != "linkme")
    {
        return;
    }

    let segments = path.segments.clone();
    path.segments = [syn::PathSegment {
        ident: syn::Ident::new("rorm", path.span()),
        arguments: syn::PathArguments::None,
    }]
    .into_iter()
    .chain(segments)
    .collect();
}
