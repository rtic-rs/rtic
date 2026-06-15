use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub struct BackendArgs;

impl Parse for BackendArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(BackendArgs)
    }
}
