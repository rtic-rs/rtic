use proc_macro2::TokenStream as TokenStream2;

use syn::{parse, ForeignItemFn, ItemFn, Stmt};

use crate::syntax::{
    ast::{Init, InitArgs},
    parse::{self as syntax_parse, util},
};

impl InitArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        syntax_parse::init_args(tokens)
    }
}

impl Init {
    pub(crate) fn parse(args: InitArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item, false) && item.sig.inputs.len() == 1;

        let span = item.sig.ident.span();

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Ok((user_shared_struct, user_local_struct)) =
                util::type_is_init_return(&item.sig.output)
            {
                if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                    if rest.is_empty() {
                        return Ok(Init {
                            args,
                            attrs: item.attrs,
                            context,
                            name: item.sig.ident,
                            stmts: item.block.stmts,
                            user_shared_struct,
                            user_local_struct,
                            is_extern: false,
                        });
                    }
                }
            }
        }

        Err(parse::Error::new(
            span,
            format!(
                "the `#[init]` function must have signature `fn({name}::Context) -> (Shared resources struct, Local resources struct)`"
            ),
        ))
    }

    pub(crate) fn parse_foreign(args: InitArgs, item: ForeignItemFn) -> parse::Result<Self> {
        let valid_signature =
            util::check_foreign_fn_signature(&item, false) && item.sig.inputs.len() == 1;

        let span = item.sig.ident.span();

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Ok((user_shared_struct, user_local_struct)) =
                util::type_is_init_return(&item.sig.output)
            {
                if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                    if rest.is_empty() {
                        return Ok(Init {
                            args,
                            attrs: item.attrs,
                            context,
                            name: item.sig.ident,
                            stmts: Vec::<Stmt>::new(),
                            user_shared_struct,
                            user_local_struct,
                            is_extern: true,
                        });
                    }
                }
            }
        }

        Err(parse::Error::new(
            span,
            format!(
                "the `#[init]` function must have signature `fn({name}::Context) -> (Shared resources struct, Local resources struct)`"
            ),
        ))
    }
}
