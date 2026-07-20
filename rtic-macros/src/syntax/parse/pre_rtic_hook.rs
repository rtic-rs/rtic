use syn::{parse, ForeignItemFn, ItemFn, Stmt};

use crate::syntax::{ast::PreRticHook, parse::util};

impl PreRticHook {
    pub(crate) fn parse(item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item, false)
            && item.sig.inputs.len() == 0
            && util::type_is_unit(&item.sig.output);

        if valid_signature {
            return Ok(PreRticHook {
                attrs: item.attrs,
                name: item.sig.ident,
                stmts: item.block.stmts,
                is_extern: false,
            });
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            format!("this `#[pre_rtic_hook]` function must have signature `fn()`"),
        ))
    }

    pub(crate) fn parse_foreign(item: ForeignItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_foreign_fn_signature(&item, false)
            && item.sig.inputs.len() == 0
            && util::type_is_unit(&item.sig.output);

        if valid_signature {
            return Ok(PreRticHook {
                attrs: item.attrs,
                name: item.sig.ident,
                stmts: Vec::<Stmt>::new(),
                is_extern: true,
            });
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            format!("this `#[pre_rtic_hook]` function must have signature `fn()`"),
        ))
    }
}
