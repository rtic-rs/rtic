use syn::{parse, Field};

use crate::syntax::parse::util::FilterAttrs;
use crate::syntax::{
    ast::{LocalResource, SharedResource, SharedResourceProperties},
    parse::util,
};

impl SharedResource {
    pub(crate) fn parse(item: &Field) -> parse::Result<Self> {
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
            vis: item.vis.clone(),
        })
    }
}

impl LocalResource {
    pub(crate) fn parse(item: &Field) -> parse::Result<Self> {
        let FilterAttrs { cfgs, attrs, docs } = util::filter_attributes(item.attrs.clone());

        Ok(LocalResource {
            cfgs,
            attrs,
            docs,
            ty: Box::new(item.ty.clone()),
            vis: item.vis.clone(),
        })
    }
}
