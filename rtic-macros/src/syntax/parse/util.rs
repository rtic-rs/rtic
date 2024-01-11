use syn::{
    bracketed,
    parse::{self, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Abi, AttrStyle, Attribute, Expr, ExprPath, FnArg, ForeignItemFn, Ident, ItemFn, Pat, PatType,
    Path, PathArguments, ReturnType, Token, Type, Visibility,
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
    attr.style == AttrStyle::Outer && attr.path().segments.len() == 1 && {
        let segment = attr.path().segments.first().unwrap();
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
    let input;
    bracketed!(input in content);

    let mut resources = Map::new();

    let error_msg_no_local_resources =
        "malformed, expected 'local = [EXPRPATH: TYPE = EXPR]', or 'local = [EXPRPATH, ...]'";

    loop {
        if input.is_empty() {
            break;
        }
        // Type ascription is de-RFCd from Rust in
        // https://github.com/rust-lang/rfcs/pull/3307
        // Manually pull out the tokens

        // Two acceptable variants:
        //
        // Task local and declared (initialized in place)
        // local = [EXPRPATH: TYPE = EXPR, ...]
        //          ~~~~~~~~~~~~~~~~~~~~~~
        // or
        // Task local but not initialized
        // local = [EXPRPATH, ...],
        //          ~~~~~~~~~

        // Common: grab first identifier EXPRPATH
        // local = [EXPRPATH: TYPE = EXPR, ...]
        //          ~~~~~~~~
        let exprpath: ExprPath = input.parse()?;

        let name = extract_resource_name_ident(exprpath.path)?;

        // Extract attributes
        let ExprPath { attrs, .. } = exprpath;
        let (cfgs, attrs) = {
            let FilterAttrs { cfgs, attrs, .. } = filter_attributes(attrs);
            (cfgs, attrs)
        };

        let local;

        // Declared requries type ascription
        if input.peek(Token![:]) {
            // Handle colon
            let _: Token![:] = input.parse()?;

            // Extract the type
            let ty: Box<Type> = input.parse()?;

            if input.peek(Token![=]) {
                // Handle equal sign
                let _: Token![=] = input.parse()?;
            } else {
                return Err(parse::Error::new(
                    name.span(),
                    "malformed, expected 'IDENT: TYPE = EXPR'",
                ));
            }

            // Grab the final expression right of equal
            let expr: Box<Expr> = input.parse()?;

            // We got a trailing colon, remove it
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }

            // Error check
            match &*ty {
                Type::Array(_) => {}
                Type::Path(_) => {}
                Type::Ptr(_) => {}
                Type::Tuple(_) => {}
                _ => {
                    return Err(parse::Error::new(
                        ty.span(),
                        "unsupported type, must be an array, tuple, pointer or type path",
                    ))
                }
            };

            local = TaskLocal::Declared(Local {
                attrs,
                cfgs,
                ty,
                expr,
            });
        } else if input.peek(Token![=]) {
            // Missing type ascription is not valid
            return Err(parse::Error::new(name.span(), "malformed, expected a type"));
        } else if input.peek(Token![,]) {
            // Attributes not supported on non-initialized local resources!

            if !attrs.is_empty() {
                return Err(parse::Error::new(
                    name.span(),
                    "attributes are not supported here",
                ));
            }

            // Remove comma
            let _: Token![,] = input.parse()?;

            // Expected when multiple local resources
            local = TaskLocal::External;
        } else if input.is_empty() {
            // There was only one single local resource
            // Task local but not initialized
            // local = [EXPRPATH],
            //          ~~~~~~~~
            local = TaskLocal::External;
        } else {
            // Specifying local without any resources is invalid
            return Err(parse::Error::new(name.span(), error_msg_no_local_resources));
        };

        if resources.contains_key(&name) {
            return Err(parse::Error::new(
                name.span(),
                "resource appears more than once in list",
            ));
        }

        resources.insert(name, local);
    }

    if resources.is_empty() {
        return Err(parse::Error::new(
            input.span(),
            error_msg_no_local_resources,
        ));
    }

    Ok(resources)
}

type ParseInputResult = Option<(Box<Pat>, Result<Vec<PatType>, FnArg>)>;

pub fn parse_inputs(inputs: Punctuated<FnArg, Token![,]>, name: &str) -> ParseInputResult {
    let mut inputs = inputs.into_iter();

    match inputs.next() {
        Some(FnArg::Typed(first)) => {
            if type_is_path(&first.ty, &[name, "Context"]) {
                let rest = inputs
                    .map(|arg| match arg {
                        FnArg::Typed(arg) => Ok(arg),
                        _ => Err(arg),
                    })
                    .collect::<Result<Vec<_>, _>>();

                Some((first.pat, rest))
            } else {
                None
            }
        }

        _ => None,
    }
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
