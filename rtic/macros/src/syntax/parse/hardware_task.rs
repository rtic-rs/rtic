use syn::{parse, ForeignItemFn, ItemFn, Stmt};

use crate::syntax::parse::util::FilterAttrs;
use crate::syntax::{
    ast::{HardwareTask, HardwareTaskArgs},
    parse::util,
};

impl HardwareTask {
    pub(crate) fn parse(args: HardwareTaskArgs, item: ItemFn) -> parse::Result<Self> {
        let span = item.sig.ident.span();
        let valid_signature = util::check_fn_signature(&item, false)
            && item.sig.inputs.len() == 1
            && util::type_is_unit(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs);

                    return Ok(HardwareTask {
                        args,
                        cfgs,
                        attrs,
                        context,
                        stmts: item.block.stmts,
                        is_extern: false,
                    });
                }
            }
        }

        Err(parse::Error::new(
            span,
            format!("this task handler must have type signature `fn({name}::Context)`"),
        ))
    }
}

impl HardwareTask {
    pub(crate) fn parse_foreign(
        args: HardwareTaskArgs,
        item: ForeignItemFn,
    ) -> parse::Result<Self> {
        let span = item.sig.ident.span();
        let valid_signature = util::check_foreign_fn_signature(&item, false)
            && item.sig.inputs.len() == 1
            && util::type_is_unit(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs);

                    return Ok(HardwareTask {
                        args,
                        cfgs,
                        attrs,
                        context,
                        stmts: Vec::<Stmt>::new(),
                        is_extern: true,
                    });
                }
            }
        }

        Err(parse::Error::new(
            span,
            format!("this task handler must have type signature `fn({name}::Context)`"),
        ))
    }
}
