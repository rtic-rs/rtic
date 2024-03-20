use syn::{
    parse::{Parse, ParseStream},
    Error, Result,
};

#[derive(Debug)]
pub struct BackendArgs();

impl Parse for BackendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Err(Error::new(
            input.span(),
            "cortex backend does not accept any arguments",
        ))
    }
}
