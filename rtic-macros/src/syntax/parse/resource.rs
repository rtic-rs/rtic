use proc_macro2::Span;
use syn::{parse, Field, Visibility};

use crate::syntax::parse::util::FilterAttrs;
use crate::syntax::{
    ast::{LocalResource, SharedResource, SharedResourceProperties},
    parse::util,
};

impl SharedResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let FilterAttrs {
            cfgs,
            mut attrs,
            docs,
        } = util::filter_attributes(item.attrs.clone());

        let lock_free = util::extract_lock_free(&mut attrs)?;

        Ok(SharedResource {
            cfgs,
            attrs,
            docs,
            ty: Box::new(item.ty.clone()),
            properties: SharedResourceProperties { lock_free },
        })
    }
}

impl LocalResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let FilterAttrs { cfgs, attrs, docs } = util::filter_attributes(item.attrs.clone());

        Ok(LocalResource {
            cfgs,
            attrs,
            docs,
            ty: Box::new(item.ty.clone()),
        })
    }
}
