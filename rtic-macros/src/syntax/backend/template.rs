use syn::{
    parse::{Parse, ParseStream},
    Result,
};

#[derive(Debug)]
pub struct BackendArgs {
    // Define your backend-specific input here
}

impl Parse for BackendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!("define how to parse your backend-specific arguments")
    }
}
