use syn::{
    parse::{Parse, ParseStream},
    Result,
};

#[derive(Debug)]
pub struct BackendArgs {
    #[cfg(feature = "riscv-clint")]
    pub hart_id: syn::Ident,
}

impl Parse for BackendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            #[cfg(feature = "riscv-clint")]
            () => {
                let hart_id = input.parse()?;
                Ok(BackendArgs { hart_id })
            }
            #[cfg(feature = "riscv-mecall")]
            () => Err(syn::Error::new(
                input.span(),
                "riscv-mecall backend does not accept any arguments",
            )),
        }
    }
}
