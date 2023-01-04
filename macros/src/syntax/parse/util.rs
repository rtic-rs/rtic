use syn::{
    bracketed,
    parse::{self, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Abi, AttrStyle, Attribute, Expr, FnArg, ForeignItemFn, Ident, ItemFn, Pat, Path, PathArguments,
    ReturnType, Token, Type, Visibility,
};

use crate::syntax::{
    ast::{Access, Local, LocalResources, SharedResources, TaskLocal},
    Map,
};

pub fn abi_is_rust(abi: &Abi) -> bool {
    match &abi.name {
        None => true,
        Some(s) => s.value() == "Rust",
    }
}

pub fn attr_eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path.segments.len() == 1 && {
        let segment = attr.path.segments.first().unwrap();
        segment.arguments == PathArguments::None && *segment.ident.to_string() == *name
    }
}

/// checks that a function signature
///
/// - has no bounds (like where clauses)
/// - is not `async`
/// - is not `const`
/// - is not `unsafe`
/// - is not generic (has no type parameters)
/// - is not variadic
/// - uses the Rust ABI (and not e.g. "C")
pub fn check_fn_signature(item: &ItemFn, allow_async: bool) -> bool {
    item.vis == Visibility::Inherited
        && item.sig.constness.is_none()
        && (item.sig.asyncness.is_none() || allow_async)
        && item.sig.abi.is_none()
        && item.sig.unsafety.is_none()
        && item.sig.generics.params.is_empty()
        && item.sig.generics.where_clause.is_none()
        && item.sig.variadic.is_none()
}

#[allow(dead_code)]
pub fn check_foreign_fn_signature(item: &ForeignItemFn, allow_async: bool) -> bool {
    item.vis == Visibility::Inherited
        && item.sig.constness.is_none()
        && (item.sig.asyncness.is_none() || allow_async)
        && item.sig.abi.is_none()
        && item.sig.unsafety.is_none()
        && item.sig.generics.params.is_empty()
        && item.sig.generics.where_clause.is_none()
        && item.sig.variadic.is_none()
}

pub struct FilterAttrs {
    pub cfgs: Vec<Attribute>,
    pub docs: Vec<Attribute>,
    pub attrs: Vec<Attribute>,
}

pub fn filter_attributes(input_attrs: Vec<Attribute>) -> FilterAttrs {
    let mut cfgs = vec![];
    let mut docs = vec![];
    let mut attrs = vec![];

    for attr in input_attrs {
        if attr_eq(&attr, "cfg") {
            cfgs.push(attr);
        } else if attr_eq(&attr, "doc") {
            docs.push(attr);
        } else {
            attrs.push(attr);
        }
    }

    FilterAttrs { cfgs, docs, attrs }
}

pub fn extract_lock_free(attrs: &mut Vec<Attribute>) -> parse::Result<bool> {
    if let Some(pos) = attrs.iter().position(|attr| attr_eq(attr, "lock_free")) {
        attrs.remove(pos);
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn parse_shared_resources(content: ParseStream<'_>) -> parse::Result<SharedResources> {
    let inner;
    bracketed!(inner in content);

    let mut resources = Map::new();
    for e in inner.call(Punctuated::<Expr, Token![,]>::parse_terminated)? {
        let err = Err(parse::Error::new(
            e.span(),
            "identifier appears more than once in list",
        ));
        let (access, path) = match e {
            Expr::Path(e) => (Access::Exclusive, e.path),

            Expr::Reference(ref r) if r.mutability.is_none() => match &*r.expr {
                Expr::Path(e) => (Access::Shared, e.path.clone()),

                _ => return err,
            },

            _ => return err,
        };

        let ident = extract_resource_name_ident(path)?;

        if resources.contains_key(&ident) {
            return Err(parse::Error::new(
                ident.span(),
                "resource appears more than once in list",
            ));
        }

        resources.insert(ident, access);
    }

    Ok(resources)
}

fn extract_resource_name_ident(path: Path) -> parse::Result<Ident> {
    if path.leading_colon.is_some()
        || path.segments.len() != 1
        || path.segments[0].arguments != PathArguments::None
    {
        Err(parse::Error::new(
            path.span(),
            "resource must be an identifier, not a path",
        ))
    } else {
        Ok(path.segments[0].ident.clone())
    }
}

pub fn parse_local_resources(content: ParseStream<'_>) -> parse::Result<LocalResources> {
    let inner;
    bracketed!(inner in content);

    let mut resources = Map::new();

    for e in inner.call(Punctuated::<Expr, Token![,]>::parse_terminated)? {
        let err = Err(parse::Error::new(
            e.span(),
            "identifier appears more than once in list",
        ));

        let (name, local) = match e {
            // local = [IDENT],
            Expr::Path(path) => {
                if !path.attrs.is_empty() {
                    return Err(parse::Error::new(
                        path.span(),
                        "attributes are not supported here",
                    ));
                }

                let ident = extract_resource_name_ident(path.path)?;
                // let (cfgs, attrs) = extract_cfgs(path.attrs);

                (ident, TaskLocal::External)
            }

            // local = [IDENT: TYPE = EXPR]
            Expr::Assign(e) => {
                let (name, ty, cfgs, attrs) = match *e.left {
                    Expr::Type(t) => {
                        // Extract name and attributes
                        let (name, cfgs, attrs) = match *t.expr {
                            Expr::Path(path) => {
                                let name = extract_resource_name_ident(path.path)?;
                                let FilterAttrs { cfgs, attrs, .. } = filter_attributes(path.attrs);

                                (name, cfgs, attrs)
                            }
                            _ => return err,
                        };

                        let ty = t.ty;

                        // Error check
                        match &*ty {
                            Type::Array(_) => {}
                            Type::Path(_) => {}
                            Type::Ptr(_) => {}
                            Type::Tuple(_) => {}
                            _ => return Err(parse::Error::new(
                                ty.span(),
                                "unsupported type, must be an array, tuple, pointer or type path",
                            )),
                        };

                        (name, ty, cfgs, attrs)
                    }
                    e => return Err(parse::Error::new(e.span(), "malformed, expected a type")),
                };

                let expr = e.right; // Expr

                (
                    name,
                    TaskLocal::Declared(Local {
                        attrs,
                        cfgs,
                        ty,
                        expr,
                    }),
                )
            }

            expr => {
                return Err(parse::Error::new(
                    expr.span(),
                    "malformed, expected 'IDENT: TYPE = EXPR'",
                ))
            }
        };

        resources.insert(name, local);
    }

    Ok(resources)
}

pub fn parse_inputs(inputs: Punctuated<FnArg, Token![,]>, name: &str) -> Option<Box<Pat>> {
    let mut inputs = inputs.into_iter();

    match inputs.next() {
        Some(FnArg::Typed(first)) => {
            if type_is_path(&first.ty, &[name, "Context"]) {
                // No more inputs
                if inputs.next().is_none() {
                    return Some(first.pat);
                }
            }
        }

        _ => {}
    }

    None
}

pub fn type_is_bottom(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        matches!(**ty, Type::Never(_))
    } else {
        false
    }
}

fn extract_init_resource_name_ident(ty: Type) -> Result<Ident, ()> {
    match ty {
        Type::Path(path) => {
            let path = path.path;

            if path.leading_colon.is_some()
                || path.segments.len() != 1
                || path.segments[0].arguments != PathArguments::None
            {
                Err(())
            } else {
                Ok(path.segments[0].ident.clone())
            }
        }
        _ => Err(()),
    }
}

/// Checks Init's return type, return the user provided types for analysis
pub fn type_is_init_return(ty: &ReturnType) -> Result<(Ident, Ident), ()> {
    match ty {
        ReturnType::Default => Err(()),

        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(t) => {
                // return should be:
                // fn -> (User's #[shared] struct, User's #[local] struct)
                //
                // We check the length and the last one here, analysis checks that the user
                // provided structs are correct.
                if t.elems.len() == 2 {
                    return Ok((
                        extract_init_resource_name_ident(t.elems[0].clone())?,
                        extract_init_resource_name_ident(t.elems[1].clone())?,
                    ));
                }

                Err(())
            }

            _ => Err(()),
        },
    }
}

pub fn type_is_path(ty: &Type, segments: &[&str]) -> bool {
    match ty {
        Type::Path(tpath) if tpath.qself.is_none() => {
            tpath.path.segments.len() == segments.len()
                && tpath
                    .path
                    .segments
                    .iter()
                    .zip(segments)
                    .all(|(lhs, rhs)| lhs.ident == **rhs)
        }

        _ => false,
    }
}

pub fn type_is_unit(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        if let Type::Tuple(ref tuple) = **ty {
            tuple.elems.is_empty()
        } else {
            false
        }
    } else {
        true
    }
}
