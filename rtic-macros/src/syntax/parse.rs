mod app;
mod hardware_task;
mod idle;
mod init;
mod resource;
mod software_task;
mod util;

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream, Parser},
    token::Brace,
    Ident, Item, LitInt, Token,
};

use crate::syntax::{
    ast::{App, AppArgs, HardwareTaskArgs, IdleArgs, InitArgs, SoftwareTaskArgs, TaskLocal},
    Either,
};

// Parse the app, both app arguments and body (input)
pub fn app(args: TokenStream2, input: TokenStream2) -> parse::Result<App> {
    let args = AppArgs::parse(args)?;
    let input: Input = syn::parse2(input)?;

    App::parse(args, input)
}

pub(crate) struct Input {
    _mod_token: Token![mod],
    pub ident: Ident,
    _brace_token: Brace,
    pub items: Vec<Item>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        fn parse_items(input: ParseStream<'_>) -> parse::Result<Vec<Item>> {
            let mut items = vec![];

            while !input.is_empty() {
                items.push(input.parse()?);
            }

            Ok(items)
        }

        let content;

        let _mod_token = input.parse()?;
        let ident = input.parse()?;
        let _brace_token = braced!(content in input);
        let items = content.call(parse_items)?;

        Ok(Input {
            _mod_token,
            ident,
            _brace_token,
            items,
        })
    }
}

fn init_args(tokens: TokenStream2) -> parse::Result<InitArgs> {
    (|input: ParseStream<'_>| -> parse::Result<InitArgs> {
        if input.is_empty() {
            return Ok(InitArgs::default());
        }

        let mut local_resources = None;

        let content;
        parenthesized!(content in input);

        if !content.is_empty() {
            loop {
                // Parse identifier name
                let ident: Ident = content.parse()?;
                // Handle equal sign
                let _: Token![=] = content.parse()?;

                match &*ident.to_string() {
                    "local" => {
                        if local_resources.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        local_resources = Some(util::parse_local_resources(&content)?);
                    }
                    _ => {
                        return Err(parse::Error::new(ident.span(), "unexpected argument"));
                    }
                }

                if content.is_empty() {
                    break;
                }
                // Handle comma: ,
                let _: Token![,] = content.parse()?;
            }
        }

        if let Some(locals) = &local_resources {
            for (ident, task_local) in locals {
                if let TaskLocal::External = task_local {
                    return Err(parse::Error::new(
                        ident.span(),
                        "only declared local resources are allowed in init",
                    ));
                }
            }
        }

        Ok(InitArgs {
            local_resources: local_resources.unwrap_or_default(),
        })
    })
    .parse2(tokens)
}

fn idle_args(tokens: TokenStream2) -> parse::Result<IdleArgs> {
    (|input: ParseStream<'_>| -> parse::Result<IdleArgs> {
        if input.is_empty() {
            return Ok(IdleArgs::default());
        }

        let mut shared_resources = None;
        let mut local_resources = None;

        let content;
        parenthesized!(content in input);
        if !content.is_empty() {
            loop {
                // Parse identifier name
                let ident: Ident = content.parse()?;
                // Handle equal sign
                let _: Token![=] = content.parse()?;

                match &*ident.to_string() {
                    "shared" => {
                        if shared_resources.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        shared_resources = Some(util::parse_shared_resources(&content)?);
                    }

                    "local" => {
                        if local_resources.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        local_resources = Some(util::parse_local_resources(&content)?);
                    }

                    _ => {
                        return Err(parse::Error::new(ident.span(), "unexpected argument"));
                    }
                }
                if content.is_empty() {
                    break;
                }

                // Handle comma: ,
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(IdleArgs {
            shared_resources: shared_resources.unwrap_or_default(),
            local_resources: local_resources.unwrap_or_default(),
        })
    })
    .parse2(tokens)
}

fn task_args(tokens: TokenStream2) -> parse::Result<Either<HardwareTaskArgs, SoftwareTaskArgs>> {
    (|input: ParseStream<'_>| -> parse::Result<Either<HardwareTaskArgs, SoftwareTaskArgs>> {
        if input.is_empty() {
            return Ok(Either::Right(SoftwareTaskArgs::default()));
        }

        let mut binds = None;
        let mut priority = None;
        let mut shared_resources = None;
        let mut local_resources = None;
        let mut prio_span = None;

        let content;
        parenthesized!(content in input);
        loop {
            if content.is_empty() {
                break;
            }

            // Parse identifier name
            let ident: Ident = content.parse()?;
            let ident_s = ident.to_string();

            // Handle equal sign
            let _: Token![=] = content.parse()?;

            match &*ident_s {
                "binds" => {
                    if binds.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    // Parse identifier name
                    let ident = content.parse()?;

                    binds = Some(ident);
                }

                "priority" => {
                    if priority.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    // #lit
                    let lit: LitInt = content.parse()?;

                    if !lit.suffix().is_empty() {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be unsuffixed",
                        ));
                    }

                    let value = lit.base10_parse::<u8>().ok();
                    if value.is_none() {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be in the range 0...255",
                        ));
                    }

                    prio_span = Some(lit.span());
                    priority = Some(value.unwrap());
                }

                "shared" => {
                    if shared_resources.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    shared_resources = Some(util::parse_shared_resources(&content)?);
                }

                "local" => {
                    if local_resources.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    local_resources = Some(util::parse_local_resources(&content)?);
                }

                _ => {
                    return Err(parse::Error::new(ident.span(), "unexpected argument"));
                }
            }

            if content.is_empty() {
                break;
            }

            // Handle comma: ,
            let _: Token![,] = content.parse()?;
        }
        let shared_resources = shared_resources.unwrap_or_default();
        let local_resources = local_resources.unwrap_or_default();

        Ok(if let Some(binds) = binds {
            // Hardware tasks can't run at anything lower than 1
            let priority = priority.unwrap_or(1);

            if priority == 0 {
                return Err(parse::Error::new(
                    prio_span.unwrap(),
                    "hardware tasks are not allowed to be at priority 0",
                ));
            }

            Either::Left(HardwareTaskArgs {
                binds,
                priority,
                shared_resources,
                local_resources,
            })
        } else {
            // Software tasks start at idle priority
            let priority = priority.unwrap_or(0);

            Either::Right(SoftwareTaskArgs {
                priority,
                shared_resources,
                local_resources,
            })
        })
    })
    .parse2(tokens)
}
