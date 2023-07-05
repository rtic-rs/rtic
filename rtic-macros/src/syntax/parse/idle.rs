use proc_macro2::TokenStream as TokenStream2;
use syn::{parse, ForeignItemFn, ItemFn, Stmt};

use crate::syntax::{
    ast::{Idle, IdleArgs},
    parse::util,
};

impl IdleArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        crate::syntax::parse::idle_args(tokens)
    }
}

impl Idle {
    pub(crate) fn parse(args: IdleArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item, false)
            && item.sig.inputs.len() == 1
            && util::type_is_bottom(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        name: item.sig.ident,
                        stmts: item.block.stmts,
                        is_extern: false,
                    });
                }
            }
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            format!("this `#[idle]` function must have signature `fn({name}::Context) -> !`"),
        ))
    }

    pub(crate) fn parse_foreign(args: IdleArgs, item: ForeignItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_foreign_fn_signature(&item, false)
            && item.sig.inputs.len() == 1
            && util::type_is_bottom(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        name: item.sig.ident,
                        stmts: Vec::<Stmt>::new(),
                        is_extern: true,
                    });
                }
            }
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            format!("this `#[idle]` function must have signature `fn({name}::Context) -> !`"),
        ))
    }
}
