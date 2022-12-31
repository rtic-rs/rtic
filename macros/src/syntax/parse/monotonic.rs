use proc_macro2::Span;
use syn::Attribute;
use syn::{parse, spanned::Spanned, ItemType, Visibility};

use crate::syntax::parse::util::FilterAttrs;
use crate::syntax::{
    ast::{Monotonic, MonotonicArgs},
    parse::util,
};

impl MonotonicArgs {
    pub(crate) fn parse(attr: Attribute) -> parse::Result<Self> {
        crate::syntax::parse::monotonic_args(attr.path, attr.tokens)
    }
}

impl Monotonic {
    pub(crate) fn parse(args: MonotonicArgs, item: &ItemType, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs.clone());

        if !attrs.is_empty() {
            return Err(parse::Error::new(
                attrs[0].path.span(),
                "Monotonic does not support attributes other than `#[cfg]`",
            ));
        }

        Ok(Monotonic {
            cfgs,
            ident: item.ident.clone(),
            ty: item.ty.clone(),
            args,
        })
    }
}
