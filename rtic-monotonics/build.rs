fn main() {
    #[cfg(feature = "stm32-metapac")]
    stm32();

    println!("cargo::rustc-check-cfg=cfg(stm32)");
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(feature = "stm32-metapac")]
fn stm32() {
    use std::path::PathBuf;
    use std::{env, fs};

    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    use stm32_metapac::metadata::METADATA;
    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| {
            !x.starts_with("CARGO_FEATURE_STM32_METAPAC")
                && !x.starts_with("CARGO_FEATURE_STM32_TIM")
                && x.starts_with("CARGO_FEATURE_STM32")
        })
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No stm32xx Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple stm32xx Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase();

    // Allows to just use #[cfg(stm32)] if one of the stm32 chips is used.
    println!("cargo:rustc-cfg=stm32");

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            println!("cargo:rustc-cfg={}", r.kind);
            println!("cargo:rustc-cfg={}_{}", r.kind, r.version);
        }
    }

    // ========
    // Generate singletons

    let mut singletons: Vec<String> = Vec::new();
    for p in METADATA.peripherals {
        if !p.name.contains("TIM") {
            continue;
        }
        if let Some(r) = &p.registers {
            match r.kind {
                // Generate singletons per pin, not per port
                "gpio" => {
                    println!("{}", p.name);
                    let port_letter = p.name.strip_prefix("GPIO").unwrap();
                    for pin_num in 0..16 {
                        singletons.push(format!("P{port_letter}{pin_num}"));
                    }
                }

                // No singleton for these, the HAL handles them specially.
                "exti" => {}

                // We *shouldn't* have singletons for these, but the HAL currently requires
                // singletons, for using with RccPeripheral to enable/disable clocks to them.
                "rcc" => {
                    if r.version.starts_with("h5")
                        || r.version.starts_with("h7")
                        || r.version.starts_with("f4")
                    {
                        singletons.push("MCO1".to_string());
                        singletons.push("MCO2".to_string());
                    }
                    if r.version.starts_with("l4") {
                        singletons.push("MCO".to_string());
                    }
                    singletons.push(p.name.to_string());
                }
                //"dbgmcu" => {}
                //"syscfg" => {}
                //"dma" => {}
                //"bdma" => {}
                //"dmamux" => {}

                // For other peripherals, one singleton per peri
                _ => singletons.push(p.name.to_string()),
            }
        }
    }

    let mut g = TokenStream::new();

    // ========
    // Generate RccPeripheral impls

    for p in METADATA.peripherals {
        if !singletons.contains(&p.name.to_string()) {
            continue;
        }

        if let Some(rcc) = &p.rcc {
            let en = rcc.enable.as_ref().unwrap();

            let rst = match &rcc.reset {
                Some(rst) => {
                    let rst_reg = format_ident!("{}", rst.register.to_ascii_lowercase());
                    let set_rst_field = format_ident!("set_{}", rst.field.to_ascii_lowercase());
                    quote! {
                        stm32_metapac::RCC.#rst_reg().modify(|w| w.#set_rst_field(true));
                        stm32_metapac::RCC.#rst_reg().modify(|w| w.#set_rst_field(false));
                    }
                }
                None => TokenStream::new(),
            };

            let after_enable = if chip_name.starts_with("stm32f2") {
                // Errata: ES0005 - 2.1.11 Delay after an RCC peripheral clock enabling
                quote! {
                    cortex_m::asm::dsb();
                }
            } else {
                TokenStream::new()
            };

            let pname = format_ident!("{}", p.name);
            let en_reg = format_ident!("{}", en.register.to_ascii_lowercase());
            let set_en_field = format_ident!("set_{}", en.field.to_ascii_lowercase());

            g.extend(quote! {
                #[doc(hidden)]
                pub mod #pname {
                    pub fn enable() {
                        stm32_metapac::RCC.#en_reg().modify(|w| w.#set_en_field(true));
                        #after_enable
                    }
                    pub fn reset() {
                        #rst
                    }
                }
            });
        }
    }

    // ========
    // Generate NVIC impl
    let prio_bits = METADATA.nvic_priority_bits;
    g.extend(quote! {
        pub const NVIC_PRIO_BITS: u8 = #prio_bits;
    });

    // ========
    // Write generated.rs

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(out_file, g.to_string()).unwrap();
}

#[cfg(feature = "stm32-metapac")]
enum GetOneError {
    None,
    Multiple,
}

#[cfg(feature = "stm32-metapac")]
trait IteratorExt: Iterator {
    fn get_one(self) -> Result<Self::Item, GetOneError>;
}

#[cfg(feature = "stm32-metapac")]
impl<T: Iterator> IteratorExt for T {
    fn get_one(mut self) -> Result<Self::Item, GetOneError> {
        match self.next() {
            None => Err(GetOneError::None),
            Some(res) => match self.next() {
                Some(_) => Err(GetOneError::Multiple),
                None => Ok(res),
            },
        }
    }
}
