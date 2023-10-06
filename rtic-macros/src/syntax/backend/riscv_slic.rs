use syn::{
    parse::{Parse, ParseStream},
    Ident, Result,
};

#[derive(Debug)]
pub struct BackendArgs {
    pub hart_id: Ident,
}

impl Parse for BackendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let hart_id = input.parse()?;
        Ok(BackendArgs { hart_id })
    }
}
